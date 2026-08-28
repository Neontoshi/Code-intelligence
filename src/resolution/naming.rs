use crate::parser::tree_sitter::FunctionInfo;
use crate::resolution::symbol::{FileId, ModuleId, ScopeId, SymbolId, TypeId};

pub fn file_id(file_path: &str) -> FileId {
    FileId(file_path.to_string())
}

pub fn module_id_for_file(file_path: &str) -> ModuleId {
    ModuleId(file_to_module_path(file_path))
}

pub fn file_scope_id(file_path: &str) -> ScopeId {
    ScopeId(format!("file_{}", file_path))
}

pub fn container_scope_id(file_path: &str, container: &str) -> ScopeId {
    ScopeId(format!("container_{}::{}", file_path, container))
}

pub fn function_symbol_id(file_path: &str, container: Option<&str>, name: &str) -> SymbolId {
    match container {
        Some(container) => SymbolId(format!("{}::{}::{}", file_path, container, name)),
        None => SymbolId(format!("{}::{}", file_path, name)),
    }
}

pub fn function_symbol_id_from_info(file_path: &str, func: &FunctionInfo) -> SymbolId {
    function_symbol_id(file_path, func.container.as_deref(), &func.name)
}

pub fn container_symbol_id(file_path: &str, container: &str) -> SymbolId {
    SymbolId(format!("{}::{}", file_path, container))
}

pub fn type_id(name: &str) -> TypeId {
    TypeId(name.to_string())
}

pub fn type_symbol_id(file_path: &str, type_name: &str) -> SymbolId {
    container_symbol_id(file_path, type_name)
}

pub fn function_scope_id(symbol_id: &SymbolId) -> ScopeId {
    ScopeId(format!("scope_{}", symbol_id.0))
}

pub fn parameter_symbol_id(function_symbol: &SymbolId, param_name: &str) -> SymbolId {
    SymbolId(format!("{}::param::{}", function_symbol.0, param_name))
}

pub fn local_variable_symbol_id(function_symbol: &SymbolId, variable_name: &str) -> SymbolId {
    SymbolId(format!("{}::local::{}", function_symbol.0, variable_name))
}

pub fn file_to_module_path(file_path: &str) -> String {
    let p = file_path.trim_start_matches("./");
    let rel = p.strip_prefix("src/").unwrap_or(p);

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
