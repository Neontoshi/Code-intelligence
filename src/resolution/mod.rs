// src/resolution/mod.rs

pub mod call_site;
pub mod context;
pub mod generic;
pub mod index_builder;
pub mod languages;
pub mod naming;
pub mod result;
pub mod scope;
pub mod symbol;
pub mod traits;
pub mod type_inference;

pub use call_site::{CallKind, CallSite, CalleeExpr, SourceLocation};
pub use context::{Language, ResolutionContext};
pub use index_builder::IndexBuilder;
pub use result::{
    ResolutionCandidate, ResolutionEvidence, ResolutionMethod, ResolutionResult, ResolutionStatus,
};
pub use scope::{Scope, ScopeChain};
pub use symbol::{
    FileId, ImportBinding, ImportKind, Module, ModuleId, ScopeId, Symbol, SymbolId, SymbolIndex,
    SymbolKind, TypeId, Visibility,
};
pub use traits::LanguageResolver;
pub use type_inference::{InferredType, TypeContext};

use std::collections::HashMap;
use std::sync::Arc;

pub struct ResolutionEngine {
    pub index: SymbolIndex,
    pub scopes: ScopeChain,
    pub resolvers: HashMap<Language, Arc<dyn LanguageResolver>>,
    pub type_context: TypeContext,
}

impl ResolutionEngine {
    pub fn new() -> Self {
        Self {
            index: SymbolIndex::new(),
            scopes: ScopeChain::new(),
            resolvers: languages::get_all_resolvers(),
            type_context: TypeContext::new(),
        }
    }

    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.index.add_symbol(symbol);
    }

    pub fn add_scope(&mut self, scope: Scope) {
        self.scopes.add_scope(scope);
    }

    pub fn infer_and_resolve_call(
        &self,
        call: &CallSite,
        file: &FileId,
        function: &SymbolId,
        scope: &ScopeId,
        language: &Language,
        module: &ModuleId,
    ) -> ResolutionResult {
        // Try to resolve using type inference
        if let CalleeExpr::Member { receiver, member } = &call.callee {
            if let CalleeExpr::Name(name) = receiver.as_ref() {
                // Infer the type of the receiver, preferring variables declared in the
                // current function scope before falling back to global heuristics.
                let receiver_type = self
                    .type_context
                    .infer_type_in_context(Some(&function.0), name);

                if let Some(type_name) = receiver_type.name() {
                    // Try to find the method on this type
                    let candidates = self.find_method_on_type(type_name, member);

                    if candidates.len() == 1 {
                        return ResolutionResult::resolved(
                            candidates[0].clone(),
                            0.95,
                            ResolutionMethod::TypeMember,
                            vec![ResolutionEvidence::MatchingType],
                        );
                    }

                    // Try to find the field on this type
                    if let Some(field_type) = self.type_context.lookup_field(type_name, member) {
                        if let Some(field_type_name) = field_type.name() {
                            // If the field is a function type, we can resolve it
                            let field_candidates = self.index.find_by_name(field_type_name);
                            if field_candidates.len() == 1 {
                                return ResolutionResult::resolved(
                                    field_candidates[0].id.clone(),
                                    0.85,
                                    ResolutionMethod::TypeMember,
                                    vec![ResolutionEvidence::MatchingType],
                                );
                            }
                        }
                    }
                }
            }
        }

        // Fall back to regular resolution
        self.resolve_call(call, file, function, scope, language, module)
    }

    fn find_method_on_type(&self, type_name: &str, method_name: &str) -> Vec<SymbolId> {
        let mut results = Vec::new();

        // Search in type_files
        if let Some(files) = self.index.type_files.get(type_name) {
            for file_id in files {
                // Try direct path
                let direct = format!("{}::{}::{}", file_id.0, type_name, method_name);
                if let Some(target) = self.index.symbols.get(&SymbolId(direct)) {
                    results.push(target.id.clone());
                }

                // Try without container
                let simple = format!("{}::{}", file_id.0, method_name);
                if let Some(target) = self.index.symbols.get(&SymbolId(simple)) {
                    results.push(target.id.clone());
                }
            }
        }

        // Search by container
        for (container, members) in &self.index.by_container {
            if container.0.contains(type_name) {
                for member_id in members {
                    if let Some(symbol) = self.index.symbols.get(member_id) {
                        if symbol.name == method_name {
                            results.push(symbol.id.clone());
                        }
                    }
                }
            }
        }

        // Deduplicate
        results.sort();
        results.dedup();
        results
    }

    pub fn resolve_call(
        &self,
        call: &CallSite,
        file: &FileId,
        function: &SymbolId,
        scope: &ScopeId,
        language: &Language,
        module: &ModuleId,
    ) -> ResolutionResult {
        let context = ResolutionContext {
            file: file.clone(),
            function: function.clone(),
            scope: scope.clone(),
            language: language.clone(),
            module: module.clone(),
            index: &self.index,
            scopes: &self.scopes,
        };

        if let Some(resolver) = self.resolvers.get(language) {
            return resolver.resolve_call(call, &context);
        }

        ResolutionResult::unresolved()
    }

    pub fn is_external_call(
        &self,
        callee: &CalleeExpr,
        language: &Language,
        file: &FileId,
    ) -> bool {
        match callee {
            CalleeExpr::Name(name) => {
                if self.is_external_builtin(name, language) || self.is_external_root(name, language)
                {
                    return true;
                }

                // A bare call whose name was brought in via `use external::path::name;`
                // is external even if the name itself isn't in a hardcoded list.
                if let Some(imports) = self.index.imports.get(file) {
                    for import in imports {
                        let matches = import.local_name == *name
                            || import.imported_name.as_deref() == Some(name.as_str());
                        if matches {
                            let module_path = &import.module.0;
                            let root = module_path.split("::").next().unwrap_or(module_path);
                            if self.is_external_root(root, language)
                                || self.is_external_root(module_path, language)
                            {
                                return true;
                            }
                        }
                    }
                }

                false
            }
            CalleeExpr::Qualified(parts) => {
                if parts.is_empty() {
                    return false;
                }

                // Check if ANY prefix of the qualified path is external
                // e.g., "tempfile::NamedTempFile::new" -> check "tempfile", "tempfile::NamedTempFile"
                let mut prefix = String::new();
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        prefix.push_str("::");
                    }
                    prefix.push_str(part);

                    if self.is_external_root(&prefix, language)
                        || self.is_external_root(part, language)
                    {
                        return true;
                    }
                }

                // Also check if the first part is external
                self.is_external_root(&parts[0], language)
            }
            CalleeExpr::Member { receiver, .. } => {
                if let CalleeExpr::Name(name) = receiver.as_ref() {
                    self.is_external_root(name, language) || self.is_external_type(name, language)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn is_external_root(&self, root: &str, language: &Language) -> bool {
        // First check if this root exists in our index
        let is_in_index = self.index.symbols.values().any(|s| {
            s.id.0 == root || s.id.0.starts_with(&format!("{}::", root)) || s.name == root
        }) || self.index.modules.values().any(|m| {
            m.id.0 == root
                || m.id.0.starts_with(&format!("{}::", root))
                || m.id.0.ends_with(&format!("::{}", root))
        });

        if is_in_index {
            return false;
        }

        // Extract the first segment (before any :: or .)
        let first_segment = root
            .split("::")
            .next()
            .unwrap_or(root)
            .split('.')
            .next()
            .unwrap_or(root);

        // Language-specific external roots
        match language {
            Language::Rust => {
                matches!(
                    first_segment,
                    "std"
                        | "core"
                        | "alloc"
                        | "tokio"
                        | "tempfile"
                        | "tempdir"
                        | "serde"
                        | "serde_json"
                        | "regex"
                        | "clap"
                        | "petgraph"
                        | "futures"
                        | "async_trait"
                        | "thiserror"
                        | "anyhow"
                        | "tracing"
                        | "log"
                        | "env_logger"
                        | "rand"
                        | "chrono"
                        | "uuid"
                        | "bytes"
                        | "http"
                        | "hyper"
                        | "axum"
                        | "actix"
                        | "rocket"
                        | "warp"
                        | "tonic"
                        | "prost"
                        | "sqlx"
                        | "diesel"
                        | "bincode"
                        | "hex"
                        | "sha2"
                        | "ndarray"
                        | "reqwest"
                        | "stream"
                        | "Regex"
                        | "DefaultHasher"
                        | "DiGraph"
                        | "VecDeque"
                        | "SystemTime"
                        | "Array1"
                        | "Array2"
                        | "Vec"
                        | "String"
                        | "PathBuf"
                        | "Path"
                        | "Instant"
                        | "NamedTempFile"
                        | "Sha256"
                        | "File"
                        | "BufWriter"
                        | "BufReader"
                        | "Mutex"
                        | "Duration"
                        | "Client"
                        | "HashMap"
                        | "HashSet"
                        | "Option"
                        | "Result"
                        | "Some"
                        | "None"
                        | "Box"
                        | "Arc"
                        | "Rc"
                        | "Bump"
                        | "Blake3Hasher"
                        | "WalkDir"
                        | "walkdir"
                        | "DashMap"
                        | "AtomicUsize"
                        | "Command"
                        | "Utc"
                        | "rayon"
                        | "tracing_subscriber"
                        | "toml"
                )
            }

            Language::Python => {
                let stdlib = [
                    "os",
                    "sys",
                    "json",
                    "re",
                    "datetime",
                    "pathlib",
                    "collections",
                    "itertools",
                    "functools",
                    "typing",
                    "abc",
                    "asyncio",
                    "concurrent",
                    "contextlib",
                    "copy",
                    "csv",
                    "enum",
                    "glob",
                    "hashlib",
                    "http",
                    "io",
                    "logging",
                    "math",
                    "random",
                    "string",
                    "subprocess",
                    "tempfile",
                    "threading",
                    "time",
                    "unittest",
                    "uuid",
                    "warnings",
                    "weakref",
                    "xml",
                    "zipfile",
                ];
                stdlib.contains(&first_segment) || first_segment.contains(".")
            }

            Language::JavaScript | Language::TypeScript => {
                let builtins = [
                    "fs",
                    "path",
                    "os",
                    "http",
                    "https",
                    "crypto",
                    "stream",
                    "util",
                    "events",
                    "buffer",
                    "process",
                    "assert",
                    "child_process",
                    "cluster",
                    "dgram",
                    "dns",
                    "net",
                    "readline",
                    "tls",
                    "url",
                    "zlib",
                ];
                let common_packages = [
                    "react",
                    "react-dom",
                    "vue",
                    "angular",
                    "express",
                    "koa",
                    "next",
                    "nuxt",
                    "axios",
                    "lodash",
                    "moment",
                    "chalk",
                    "commander",
                    "dotenv",
                    "jest",
                    "mocha",
                    "chai",
                    "sinon",
                    "webpack",
                    "babel",
                    "typescript",
                    "eslint",
                ];
                builtins.contains(&first_segment)
                    || common_packages.contains(&first_segment)
                    || first_segment.starts_with("@")
                    || first_segment.starts_with("node:")
            }

            Language::Go => {
                // Go standard library packages
                first_segment.contains(".")
                    && !first_segment.starts_with("internal/")
                    && !first_segment.starts_with("local/")
                    && !first_segment.starts_with("example/")
            }
            Language::Java => {
                first_segment.starts_with("java.")
                    || first_segment.starts_with("javax.")
                    || first_segment.starts_with("org.springframework")
                    || first_segment.starts_with("org.junit")
                    || first_segment.starts_with("com.google")
                    || first_segment.starts_with("org.apache")
                    || first_segment.starts_with("org.slf4j")
                    || first_segment.starts_with("lombok")
            }
            Language::Dart => {
                first_segment.starts_with("dart:") || first_segment.starts_with("package:")
            }
            Language::Php => first_segment.starts_with("\\") || first_segment.contains("\\"),
            Language::Cpp => {
                first_segment == "std"
                    || first_segment.starts_with("std::")
                    || first_segment.starts_with("boost::")
            }
            Language::CSharp => {
                first_segment.starts_with("System.")
                    || first_segment.starts_with("Microsoft.")
                    || first_segment == "System"
                    || first_segment == "Microsoft"
            }
        }
    }

    pub fn is_external_type(&self, name: &str, language: &Language) -> bool {
        // Check if this type exists in our index
        let is_in_index =
            self.index.by_name.contains_key(name) || self.index.type_files.contains_key(name);

        if is_in_index {
            return false;
        }

        // Common external types per language
        match language {
            Language::Rust => {
                matches!(
                    name,
                    "Vec"
                        | "String"
                        | "PathBuf"
                        | "HashMap"
                        | "HashSet"
                        | "Box"
                        | "Arc"
                        | "Rc"
                        | "Mutex"
                        | "RwLock"
                        | "Option"
                        | "Result"
                        | "File"
                        | "SocketAddr"
                        | "Duration"
                        | "Instant"
                )
            }
            Language::Python => name.chars().next().map_or(false, |c| c.is_uppercase()),
            Language::JavaScript | Language::TypeScript => {
                matches!(
                    name,
                    "Promise"
                        | "Array"
                        | "Object"
                        | "String"
                        | "Number"
                        | "Boolean"
                        | "Map"
                        | "Set"
                        | "WeakMap"
                        | "WeakSet"
                        | "Date"
                        | "RegExp"
                        | "Error"
                        | "TypeError"
                        | "SyntaxError"
                        | "JSON"
                        | "Math"
                        | "Reflect"
                        | "Proxy"
                        | "Symbol"
                        | "BigInt"
                )
            }
            Language::Go => name.chars().next().map_or(false, |c| c.is_uppercase()),
            Language::Java => {
                name.starts_with("java.")
                    || name.starts_with("javax.")
                    || name.starts_with("org.")
                    || name.starts_with("com.")
            }
            Language::Cpp => name.starts_with("std::"),
            Language::CSharp => name.starts_with("System.") || name.starts_with("Microsoft."),
            _ => false,
        }
    }

    fn is_external_builtin(&self, name: &str, language: &Language) -> bool {
        match language {
            Language::Python => {
                matches!(
                    name,
                    "print"
                        | "len"
                        | "range"
                        | "enumerate"
                        | "zip"
                        | "map"
                        | "filter"
                        | "sorted"
                        | "reversed"
                        | "sum"
                        | "min"
                        | "max"
                        | "abs"
                        | "round"
                        | "isinstance"
                        | "issubclass"
                        | "hasattr"
                        | "getattr"
                        | "setattr"
                        | "delattr"
                        | "id"
                        | "type"
                        | "repr"
                        | "str"
                        | "int"
                        | "float"
                        | "bool"
                        | "list"
                        | "dict"
                        | "set"
                        | "tuple"
                        | "frozenset"
                        | "bytes"
                )
            }
            Language::JavaScript | Language::TypeScript => {
                matches!(
                    name,
                    "parseInt"
                        | "parseFloat"
                        | "isNaN"
                        | "isFinite"
                        | "encodeURI"
                        | "decodeURI"
                        | "encodeURIComponent"
                        | "decodeURIComponent"
                        | "eval"
                        | "setTimeout"
                        | "setInterval"
                        | "clearTimeout"
                        | "clearInterval"
                        | "require"
                        | "console"
                )
            }
            Language::Go => {
                matches!(
                    name,
                    "len"
                        | "cap"
                        | "make"
                        | "new"
                        | "append"
                        | "copy"
                        | "delete"
                        | "panic"
                        | "recover"
                        | "close"
                        | "print"
                        | "println"
                )
            }
            Language::Rust => {
                matches!(
                    name,
                    "Some"
                        | "None"
                        | "Ok"
                        | "Err"
                        | "vec"
                        | "format"
                        | "print"
                        | "println"
                        | "eprint"
                        | "eprintln"
                        | "panic"
                        | "assert"
                        | "assert_eq"
                        | "assert_ne"
                        | "debug_assert"
                        | "todo"
                        | "unimplemented"
                        | "unreachable"
                        | "black_box"
                )
            }
            _ => false,
        }
    }
}

impl Default for ResolutionEngine {
    fn default() -> Self {
        Self::new()
    }
}
