// src/resolution/symbol.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Ord, PartialOrd)]
pub struct SymbolId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModuleId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScopeId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub file: FileId,
    pub container: Option<SymbolId>,
    pub module: Option<ModuleId>,
    pub signature: Option<String>,
    pub visibility: Visibility,
    pub declared_type: Option<TypeId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Method,
    Constructor,
    StaticMethod,
    ClassMethod,
    TraitMethod,
    AssociatedFunction,
    Type,
    Closure,
    Callback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
    Exported,
}

#[derive(Debug, Clone, Default)]
pub struct SymbolIndex {
    pub symbols: HashMap<SymbolId, Symbol>,
    pub by_name: HashMap<String, Vec<SymbolId>>,
    pub by_qualified_name: HashMap<String, Vec<SymbolId>>,
    pub by_file: HashMap<FileId, Vec<SymbolId>>,
    pub by_container: HashMap<SymbolId, Vec<SymbolId>>,
    pub by_type: HashMap<TypeId, Vec<SymbolId>>,
    pub imports: HashMap<FileId, Vec<ImportBinding>>,
    pub modules: HashMap<ModuleId, Module>,
    pub type_files: HashMap<String, Vec<FileId>>,
    pub type_symbols: HashMap<TypeId, SymbolId>,
    pub type_name_to_id: HashMap<String, TypeId>,
    pub module_aliases: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportBinding {
    pub local_name: String,
    pub imported_name: Option<String>,
    pub module: ModuleId,
    pub symbol: Option<SymbolId>,
    pub scope: ScopeId,
    pub kind: ImportKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportKind {
    Module,
    Symbol,
    Wildcard,
    Namespace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: ModuleId,
    pub file: FileId,
    pub parent: Option<ModuleId>,
    pub children: Vec<ModuleId>,
}

impl SymbolIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_type(&mut self, name: String, file: FileId) {
        self.type_files.entry(name).or_default().push(file);
    }

    pub fn register_type_symbol(&mut self, type_id: TypeId, symbol_id: SymbolId) {
        self.type_symbols.insert(type_id.clone(), symbol_id);
        self.type_name_to_id
            .entry(type_id.0.clone())
            .or_insert(type_id);
    }

    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.by_name
            .entry(symbol.name.clone())
            .or_default()
            .push(symbol.id.clone());

        if let Some(module) = &symbol.module {
            let qualified = format!("{}::{}", module.0, symbol.name);
            self.by_qualified_name
                .entry(qualified)
                .or_default()
                .push(symbol.id.clone());
        }

        self.by_file
            .entry(symbol.file.clone())
            .or_default()
            .push(symbol.id.clone());

        if let Some(container) = &symbol.container {
            self.by_container
                .entry(container.clone())
                .or_default()
                .push(symbol.id.clone());
        }

        self.symbols.insert(symbol.id.clone(), symbol);
    }

    pub fn find_by_name(&self, name: &str) -> Vec<&Symbol> {
        self.by_name
            .get(name)
            .map(|ids| ids.iter().filter_map(|id| self.symbols.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn find_by_qualified(&self, qualified: &str) -> Vec<&Symbol> {
        self.by_qualified_name
            .get(qualified)
            .map(|ids| ids.iter().filter_map(|id| self.symbols.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn find_in_file(&self, file: &FileId, name: &str) -> Vec<&Symbol> {
        self.by_file
            .get(file)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.symbols.get(id))
                    .filter(|s| s.name == name)
                    .collect()
            })
            .unwrap_or_default()
    }
}
