// src/resolution/index_builder.rs

use crate::parser::tree_sitter::ParsedFile;
use crate::parser::TypeKind;
use crate::resolution::scope::{Scope, ScopeChain};
use crate::resolution::symbol::{
    FileId, ImportBinding, ImportKind, Module, ModuleId, ScopeId, Symbol, SymbolId, SymbolIndex,
    SymbolKind, Visibility,
};
use crate::resolution::type_inference::{InferredType, TypeContext};
use std::collections::HashMap;

pub struct IndexBuilder;

impl IndexBuilder {
    pub fn build(files: &[ParsedFile]) -> (SymbolIndex, ScopeChain, TypeContext) {
        let mut index = SymbolIndex::new();
        let mut scopes = ScopeChain::new();
        let mut type_context = TypeContext::new();

        // First pass: Register all modules and file-level scopes
        for file in files {
            let file_path = file.path.clone();
            let file_id = FileId(file_path.clone());
            let module_id = ModuleId(Self::file_to_module_path(&file_path));

            // Register module
            index.modules.insert(
                module_id.clone(),
                Module {
                    id: module_id.clone(),
                    file: file_id.clone(),
                    parent: None,
                    children: Vec::new(),
                },
            );

            // Create file-level scope
            let file_scope_id = ScopeId(format!("file_{}", file_path));
            let file_scope = Scope::new(file_scope_id.clone(), None);
            scopes.add_scope(file_scope);

            // Register imports
            let mut import_bindings = Vec::new();
            for import in &file.imports {
                // Parse import items properly
                let local_name = import
                    .items
                    .first()
                    .cloned()
                    .unwrap_or_else(|| import.module.clone());

                // Try to extract the imported name from the module path
                let imported_name = Self::extract_imported_name(&import.module);

                let binding = ImportBinding {
                    local_name: local_name.clone(),
                    imported_name,
                    module: ModuleId(import.module.clone()),
                    symbol: None,
                    scope: file_scope_id.clone(),
                    kind: Self::classify_import(&import.module),
                };
                import_bindings.push(binding);
            }

            for ty in &file.types {
                index.add_type(ty.name.clone(), file_id.clone());
            }

            index.imports.insert(file_id, import_bindings);
        }

        // Second pass: Register container scopes (class/struct/impl)
        for file in files {
            let file_path = file.path.clone();

            for func in &file.functions {
                if let Some(container) = &func.container {
                    let container_scope_id =
                        ScopeId(format!("container_{}::{}", file_path, container));
                    let file_scope_id = ScopeId(format!("file_{}", file_path));

                    if !scopes.scopes.contains_key(&container_scope_id) {
                        let container_scope =
                            Scope::new(container_scope_id.clone(), Some(file_scope_id));
                        scopes.add_scope(container_scope);
                    }
                }
            }
        }

        // Third pass: Register function symbols and scopes
        for file in files {
            let file_path = file.path.clone();
            let file_id = FileId(file_path.clone());
            let module_id = ModuleId(Self::file_to_module_path(&file_path));
            let file_scope_id = ScopeId(format!("file_{}", file_path));

            for func in &file.functions {
                let full_path = match &func.container {
                    Some(c) => format!("{}::{}::{}", file_path, c, func.name),
                    None => format!("{}::{}", file_path, func.name),
                };

                let symbol = Symbol {
                    id: SymbolId(full_path.clone()),
                    name: func.name.clone(),
                    kind: if func.is_trait_method {
                        SymbolKind::TraitMethod
                    } else if func.container.is_some() {
                        SymbolKind::Method
                    } else {
                        SymbolKind::Function
                    },
                    file: file_id.clone(),
                    container: func
                        .container
                        .as_ref()
                        .map(|c| SymbolId(format!("{}::{}", file_path, c))),
                    module: Some(module_id.clone()),
                    signature: func.return_type.clone(),
                    visibility: if func.is_public {
                        Visibility::Public
                    } else {
                        Visibility::Private
                    },
                };

                index.add_symbol(symbol);

                // Function scope with correct parent chain
                let scope_id = ScopeId(format!("scope_{}", full_path));
                let parent_scope = if let Some(container) = &func.container {
                    Some(ScopeId(format!("container_{}::{}", file_path, container)))
                } else {
                    Some(file_scope_id.clone())
                };

                let mut scope = Scope::new(scope_id, parent_scope);

                // Insert parameters into scope
                for param in &func.params {
                    scope.insert(
                        param.name.clone(),
                        SymbolId(format!("{}::param::{}", full_path, param.name)),
                    );
                }

                scopes.add_scope(scope);
            }
        }

        // Fourth pass: Resolve imports to symbols
        // This creates links between imports and actual symbols when possible
        let mut import_resolutions: Vec<(FileId, usize, SymbolId)> = Vec::new();

        for (file_id, imports) in &index.imports {
            for (idx, import) in imports.iter().enumerate() {
                // Try to resolve import module to a symbol
                let module_candidates = index.find_by_qualified(&import.module.0);

                // Also try to find by the imported name
                let name_to_find = import.imported_name.as_ref().unwrap_or(&import.local_name);
                let name_candidates = index.find_by_name(name_to_find);

                // Prefer module-qualified matches
                if !module_candidates.is_empty() {
                    import_resolutions.push((
                        file_id.clone(),
                        idx,
                        module_candidates[0].id.clone(),
                    ));
                } else if name_candidates.len() == 1 {
                    import_resolutions.push((file_id.clone(), idx, name_candidates[0].id.clone()));
                }
            }
        }

        // Fifth pass: Build type context
        for file in files {
            // Register type aliases (FileId, ModuleId, etc.)
            // These are tuple structs defined in resolution/symbol.rs
            for ty in &file.types {
                match ty.kind {
                    TypeKind::Struct => {
                        // Register as a concrete type
                        type_context.register_struct(&ty.name, HashMap::new());
                    }
                    TypeKind::Enum => {
                        type_context.register_struct(&ty.name, HashMap::new());
                    }
                    TypeKind::TypeAlias => {
                        type_context
                            .register_type_alias(&ty.name, InferredType::Concrete(ty.name.clone()));
                    }
                    _ => {}
                }
            }

            // Register function signatures
            for func in &file.functions {
                let full_path = match &func.container {
                    Some(c) => format!("{}::{}::{}", file.path, c, func.name),
                    None => format!("{}::{}", file.path, func.name),
                };

                let params: Vec<InferredType> = func
                    .params
                    .iter()
                    .map(|p| {
                        p.type_hint
                            .as_ref()
                            .map(|t| InferredType::from_string(t))
                            .unwrap_or(InferredType::Unknown)
                    })
                    .collect();

                let return_type = func
                    .return_type
                    .as_ref()
                    .map(|t| InferredType::from_string(t))
                    .unwrap_or(InferredType::Unknown);

                type_context.register_function(&full_path, params, return_type);
            }
        }

        // Apply import resolutions
        for (file_id, idx, symbol_id) in import_resolutions {
            if let Some(imports) = index.imports.get_mut(&file_id) {
                if let Some(import) = imports.get_mut(idx) {
                    import.symbol = Some(symbol_id);
                }
            }
        }

        // Build module aliases
        for (module_id, module) in &index.modules {
            let file_path = &module.file.0;

            // Register the full module path
            index
                .module_aliases
                .insert(module_id.0.clone(), file_path.clone());

            // Register the short name (last segment)
            let short_name = module_id.0.split("::").last().unwrap_or(&module_id.0);
            index
                .module_aliases
                .insert(short_name.to_string(), file_path.clone());

            // Register common aliases
            let file_stem = file_path
                .split('/')
                .last()
                .unwrap_or(file_path)
                .trim_end_matches(".rs");
            index
                .module_aliases
                .insert(file_stem.to_string(), file_path.clone());

            // Register path-based aliases (src/parser/languages/rust -> rust)
            if let Some(lang_pos) = file_path.find("/languages/") {
                let lang_name = file_path[lang_pos + 11..].trim_end_matches(".rs");
                index
                    .module_aliases
                    .insert(lang_name.to_string(), file_path.clone());
            }
        }

        (index, scopes, type_context)
    }

    fn extract_imported_name(module: &str) -> Option<String> {
        // Handle different import syntaxes:
        // Rust: "std::fs::write" -> "write"
        // Python: "from module import name" -> "name" (handled elsewhere)
        // JavaScript: "import { name } from 'module'" -> "name" (handled elsewhere)
        // Go: "import (\"fmt\")" -> "fmt"
        // Java: "import java.util.List;" -> "List"

        // Try to split on common separators
        let separators = ["::", ".", "/"];
        for sep in separators {
            if let Some(last) = module.rsplit(sep).next() {
                if !last.is_empty() {
                    return Some(last.to_string());
                }
            }
        }

        None
    }

    fn classify_import(module: &str) -> ImportKind {
        // Classify import type based on syntax
        if module.ends_with(".*") || module.ends_with("::*") {
            ImportKind::Wildcard
        } else if module.contains("::") || module.contains('.') {
            ImportKind::Symbol
        } else {
            ImportKind::Module
        }
    }

    pub fn file_to_module_path(file_path: &str) -> String {
        let p = file_path.trim_start_matches("./");
        let rel = p.strip_prefix("src/").unwrap_or(p);

        // Strip all known language extensions
        let rel = rel
            .strip_suffix(".rs")
            .or_else(|| rel.strip_suffix(".py"))
            .or_else(|| rel.strip_suffix(".ts"))
            .or_else(|| rel.strip_suffix(".tsx"))
            .or_else(|| rel.strip_suffix(".js"))
            .or_else(|| rel.strip_suffix(".jsx"))
            .or_else(|| rel.strip_suffix(".go"))
            .or_else(|| rel.strip_suffix(".java"))
            .or_else(|| rel.strip_suffix(".dart"))
            .or_else(|| rel.strip_suffix(".php"))
            .or_else(|| rel.strip_suffix(".cpp"))
            .or_else(|| rel.strip_suffix(".cc"))
            .or_else(|| rel.strip_suffix(".cxx"))
            .or_else(|| rel.strip_suffix(".hpp"))
            .or_else(|| rel.strip_suffix(".h"))
            .or_else(|| rel.strip_suffix(".cs"))
            .unwrap_or(rel);

        let mut segments: Vec<&str> = rel.split('/').collect();
        if matches!(segments.last(), Some(&"mod") | Some(&"main") | Some(&"lib")) {
            segments.pop();
        }
        if segments.is_empty() {
            "crate".to_string()
        } else {
            format!("crate::{}", segments.join("::"))
        }
    }
}
