// src/resolution/index_builder.rs

use crate::parser::tree_sitter::ParsedFile;
use crate::resolution::scope::{Scope, ScopeChain};
use crate::resolution::symbol::{
    FileId, ImportBinding, ImportKind, Module, ModuleId, ScopeId, Symbol, SymbolId, SymbolIndex,
    SymbolKind, Visibility,
};

pub struct IndexBuilder;

impl IndexBuilder {
    pub fn build(files: &[ParsedFile]) -> (SymbolIndex, ScopeChain) {
        let mut index = SymbolIndex::new();
        let mut scopes = ScopeChain::new();

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
                let binding = ImportBinding {
                    local_name: import
                        .items
                        .first()
                        .cloned()
                        .unwrap_or_else(|| import.module.clone()),
                    imported_name: None,
                    module: ModuleId(import.module.clone()),
                    symbol: None,
                    scope: file_scope_id.clone(),
                    kind: ImportKind::Module,
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
                    signature: None,
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

        (index, scopes)
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
