// src/resolution/index_builder.rs

use crate::parser::tree_sitter::ParsedFile;
use crate::parser::TypeKind;
use crate::resolution::naming;
use crate::resolution::scope::{Scope, ScopeChain};
use crate::resolution::symbol::{
    FileId, ImportBinding, ImportKind, Module, ModuleId, Symbol, SymbolId, SymbolIndex, SymbolKind,
    Visibility,
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
            let file_id = naming::file_id(&file_path);
            let module_id = naming::module_id_for_file(&file_path);

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
            let file_scope_id = naming::file_scope_id(&file_path);
            let mut file_scope = Scope::new(file_scope_id.clone(), None);

            // Register imports
            let mut import_bindings = Vec::new();
            for import in &file.imports {
                let local_name = import
                    .items
                    .first()
                    .cloned()
                    .unwrap_or_else(|| import.module.clone());

                let imported_name = Self::imported_name_from_parts(import);

                let binding = ImportBinding {
                    local_name: local_name.clone(),
                    imported_name,
                    module: ModuleId(import.module.clone()),
                    symbol: None,
                    scope: file_scope_id.clone(),
                    kind: Self::classify_import_from_parts(import),
                };
                if binding.kind != ImportKind::Wildcard {
                    file_scope.insert(
                        local_name.clone(),
                        SymbolId(format!("import::{}", local_name)),
                    );
                }
                import_bindings.push(binding);
            }

            for ty in &file.types {
                index.add_type(ty.name.clone(), file_id.clone());
            }

            scopes.add_scope(file_scope);
            index.imports.insert(file_id, import_bindings);
        }

        // Second pass: Register type/container symbols and scopes.
        for file in files {
            let file_path = file.path.clone();
            let file_id = naming::file_id(&file_path);
            let module_id = naming::module_id_for_file(&file_path);
            let file_scope_id = naming::file_scope_id(&file_path);

            for ty in &file.types {
                let type_symbol_id = naming::type_symbol_id(&file_path, &ty.name);
                let type_id = naming::type_id(&ty.name);

                if !index.symbols.contains_key(&type_symbol_id) {
                    let type_symbol = Symbol {
                        id: type_symbol_id.clone(),
                        name: ty.name.clone(),
                        kind: SymbolKind::Type,
                        file: file_id.clone(),
                        container: None,
                        module: Some(module_id.clone()),
                        signature: None,
                        visibility: Visibility::Public,
                        declared_type: Some(type_id.clone()),
                    };
                    index.add_symbol(type_symbol);
                    index.register_type_symbol(type_id, type_symbol_id.clone());
                }

                let container_scope_id = naming::container_scope_id(&file_path, &ty.name);
                if !scopes.scopes.contains_key(&container_scope_id) {
                    let container_scope =
                        Scope::new(container_scope_id, Some(file_scope_id.clone()));
                    scopes.add_scope(container_scope);
                }
            }

            for func in &file.functions {
                if let Some(container) = &func.container {
                    let container_scope_id = naming::container_scope_id(&file_path, container);
                    if !scopes.scopes.contains_key(&container_scope_id) {
                        let container_scope =
                            Scope::new(container_scope_id.clone(), Some(file_scope_id.clone()));
                        scopes.add_scope(container_scope);
                    }

                    let container_symbol_id = naming::container_symbol_id(&file_path, container);
                    if !index.symbols.contains_key(&container_symbol_id) {
                        let declared_type = index.type_name_to_id.get(container).cloned();
                        let container_symbol = Symbol {
                            id: container_symbol_id,
                            name: container.clone(),
                            kind: SymbolKind::Type,
                            file: file_id.clone(),
                            container: None,
                            module: Some(module_id.clone()),
                            signature: None,
                            visibility: Visibility::Public,
                            declared_type,
                        };
                        index.add_symbol(container_symbol);
                    }
                }
            }
        }

        // Third pass: Register function symbols and scopes
        for file in files {
            let file_path = file.path.clone();
            let file_id = naming::file_id(&file_path);
            let module_id = naming::module_id_for_file(&file_path);
            let file_scope_id = naming::file_scope_id(&file_path);

            for func in &file.functions {
                let function_id = naming::function_symbol_id_from_info(&file_path, func);

                let declared_type = func
                    .container
                    .as_ref()
                    .and_then(|container| index.type_name_to_id.get(container).cloned());

                let symbol = Symbol {
                    id: function_id.clone(),
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
                        .map(|c| naming::container_symbol_id(&file_path, c)),
                    module: Some(module_id.clone()),
                    signature: func.return_type.clone(),
                    visibility: if func.is_public {
                        Visibility::Public
                    } else {
                        Visibility::Private
                    },
                    declared_type: declared_type.clone(),
                };

                if let Some(type_id) = declared_type.clone() {
                    index
                        .by_type
                        .entry(type_id)
                        .or_default()
                        .push(function_id.clone());
                }

                index.add_symbol(symbol);

                // Function scope with correct parent chain
                let scope_id = naming::function_scope_id(&function_id);
                let parent_scope = if let Some(container) = &func.container {
                    Some(naming::container_scope_id(&file_path, container))
                } else {
                    Some(file_scope_id.clone())
                };

                let mut scope = Scope::new(scope_id, parent_scope);

                // Insert parameters into scope and seed parameter type information.
                for param in &func.params {
                    let param_symbol = naming::parameter_symbol_id(&function_id, &param.name);
                    scope.insert(param.name.clone(), param_symbol);

                    if let Some(type_hint) = param.type_hint.as_deref() {
                        type_context.register_variable_with_context(
                            Some(&function_id.0),
                            &param.name,
                            Some(type_hint),
                            None,
                        );
                    }
                }

                for variable in &func.variables {
                    let local_symbol =
                        naming::local_variable_symbol_id(&function_id, &variable.name);
                    scope.insert(variable.name.clone(), local_symbol);
                    type_context.register_variable_with_context(
                        Some(&function_id.0),
                        &variable.name,
                        variable.type_hint.as_deref(),
                        variable.initializer.as_deref(),
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
                if let Some(symbol_id) = Self::resolve_import_binding(&index, file_id, import) {
                    import_resolutions.push((file_id.clone(), idx, symbol_id));
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
                let function_id = naming::function_symbol_id_from_info(&file.path, func);

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

                type_context.register_function(&function_id.0, params, return_type);
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

            index
                .module_aliases
                .insert(module_id.0.clone(), file_path.clone());

            let short_name = module_id.0.split("::").last().unwrap_or(&module_id.0);
            index
                .module_aliases
                .insert(short_name.to_string(), file_path.clone());

            let file_stem = file_path
                .split('/')
                .last()
                .unwrap_or(file_path)
                .trim_end_matches(".rs");
            index
                .module_aliases
                .insert(file_stem.to_string(), file_path.clone());

            if let Some(lang_pos) = file_path.find("/languages/") {
                let lang_name = file_path[lang_pos + 11..].trim_end_matches(".rs");
                index
                    .module_aliases
                    .insert(lang_name.to_string(), file_path.clone());
            }

            for alias in Self::derive_local_rust_aliases(file_path) {
                index
                    .module_aliases
                    .entry(alias)
                    .or_insert(file_path.clone());
            }
        }

        (index, scopes, type_context)
    }

    fn resolve_import_binding(
        index: &SymbolIndex,
        importing_file: &FileId,
        import: &ImportBinding,
    ) -> Option<SymbolId> {
        let name_to_find = import.imported_name.as_ref().unwrap_or(&import.local_name);

        if import.kind == ImportKind::Wildcard {
            return None;
        }

        if let Some(symbol_id) = Self::resolve_rust_import_to_symbol(
            index,
            importing_file,
            &import.module.0,
            name_to_find,
        ) {
            return Some(symbol_id);
        }

        let module_candidates = index.find_by_qualified(&import.module.0);
        if module_candidates.len() == 1 {
            return Some(module_candidates[0].id.clone());
        }

        let name_candidates = index.find_by_name(name_to_find);
        if name_candidates.len() == 1 {
            return Some(name_candidates[0].id.clone());
        }

        None
    }

    fn resolve_rust_import_to_symbol(
        index: &SymbolIndex,
        importing_file: &FileId,
        module_path: &str,
        item_name: &str,
    ) -> Option<SymbolId> {
        let path_prefix = if importing_file.0.starts_with("./") {
            "./"
        } else {
            ""
        };

        let candidate_files =
            Self::candidate_rust_import_files(importing_file, module_path, path_prefix);
        for file_path in candidate_files {
            let direct = SymbolId(format!("{}::{}", file_path, item_name));
            if index.symbols.contains_key(&direct) {
                return Some(direct);
            }

            let container = SymbolId(format!("{}::{}", file_path, item_name));
            if let Some(members) = index.by_container.get(&container) {
                if members.len() == 1 {
                    return Some(members[0].clone());
                }
            }
        }

        None
    }

    fn candidate_rust_import_files(
        importing_file: &FileId,
        module_path: &str,
        path_prefix: &str,
    ) -> Vec<String> {
        let mut candidates = Vec::new();
        let normalized = module_path.trim().trim_end_matches("::*");

        if let Some(rest) = normalized.strip_prefix("crate::") {
            let rel = rest.replace("::", "/");
            candidates.push(format!("{}src/{}.rs", path_prefix, rel));
            candidates.push(format!("{}src/{}/mod.rs", path_prefix, rel));
            return candidates;
        }

        if let Some(rest) = normalized.strip_prefix("self::") {
            let base = importing_file
                .0
                .trim_start_matches("./")
                .trim_end_matches(".rs")
                .trim_end_matches("/mod");
            let rel = if rest.is_empty() {
                base.to_string()
            } else {
                format!("{}/{}", base, rest.replace("::", "/"))
            };
            candidates.push(format!("{}{}.rs", path_prefix, rel));
            candidates.push(format!("{}{}//mod.rs", path_prefix, rel).replace("//", "/"));
            return candidates;
        }

        if let Some(rest) = normalized.strip_prefix("super::") {
            let base = importing_file
                .0
                .trim_start_matches("./")
                .rsplit_once('/')
                .map(|(dir, _)| dir)
                .unwrap_or("src");
            let parent = base.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("src");
            let rel = if rest.is_empty() {
                parent.to_string()
            } else {
                format!("{}/{}", parent, rest.replace("::", "/"))
            };
            candidates.push(format!("{}{}.rs", path_prefix, rel));
            candidates.push(format!("{}{}//mod.rs", path_prefix, rel).replace("//", "/"));
            return candidates;
        }

        let rel = normalized.replace("::", "/");
        candidates.push(format!("{}src/{}.rs", path_prefix, rel));
        candidates.push(format!("{}src/{}/mod.rs", path_prefix, rel));
        candidates
    }

    fn derive_local_rust_aliases(file_path: &str) -> Vec<String> {
        let normalized = file_path.trim_start_matches("./");
        let segments: Vec<&str> = normalized.split('/').collect();
        let src_pos = match segments.iter().position(|segment| *segment == "src") {
            Some(pos) => pos,
            None => return Vec::new(),
        };

        let mut aliases = Vec::new();

        if src_pos > 0 {
            aliases.push(segments[src_pos - 1].to_string());
        }

        if src_pos + 1 < segments.len() {
            let first_after_src = segments[src_pos + 1]
                .trim_end_matches(".rs")
                .trim_end_matches("mod")
                .trim_end_matches('/');
            if !first_after_src.is_empty() && first_after_src != "main" && first_after_src != "lib"
            {
                aliases.push(first_after_src.to_string());
            }
        }

        let crate_relative = &segments[src_pos + 1..];
        if !crate_relative.is_empty() {
            let mut module_parts: Vec<String> = crate_relative
                .iter()
                .map(|part| part.trim_end_matches(".rs").to_string())
                .collect();
            if matches!(
                module_parts.last().map(|s| s.as_str()),
                Some("mod") | Some("main") | Some("lib")
            ) {
                module_parts.pop();
            }
            if !module_parts.is_empty() {
                aliases.push(module_parts.join("::"));
            }
        }

        aliases.sort();
        aliases.dedup();
        aliases
    }

    fn imported_name_from_parts(import: &crate::parser::tree_sitter::ImportInfo) -> Option<String> {
        if import.items.len() >= 2 {
            return import.items.get(1).cloned();
        }

        if let Some(first) = import.items.first() {
            if first != "*" {
                return Some(first.clone());
            }
        }

        Self::extract_imported_name(&import.module)
    }

    fn extract_imported_name(module: &str) -> Option<String> {
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

    fn classify_import_from_parts(import: &crate::parser::tree_sitter::ImportInfo) -> ImportKind {
        if import.items.iter().any(|item| item == "*")
            || import.module.ends_with(".*")
            || import.module.ends_with("::*")
        {
            ImportKind::Wildcard
        } else if !import.items.is_empty()
            || import.module.contains("::")
            || import.module.contains('.')
        {
            ImportKind::Symbol
        } else {
            ImportKind::Module
        }
    }

    pub fn file_to_module_path(file_path: &str) -> String {
        naming::file_to_module_path(file_path)
    }
}
