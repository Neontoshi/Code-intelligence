// src/resolution/type_inference.rs
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InferredType {
    Concrete(String),
    Generic(String, Vec<InferredType>),
    Reference(Box<InferredType>),
    Unknown,
}

impl InferredType {
    pub fn name(&self) -> Option<&str> {
        match self {
            InferredType::Concrete(name) => Some(name),
            InferredType::Generic(name, _) => Some(name),
            InferredType::Reference(inner) => inner.name(),
            InferredType::Unknown => None,
        }
    }

    pub fn from_string(s: &str) -> Self {
        let trimmed = s.trim();

        // Handle references
        if let Some(inner) = trimmed.strip_prefix('&') {
            return InferredType::Reference(Box::new(InferredType::from_string(inner)));
        }
        if let Some(inner) = trimmed.strip_prefix("&mut ") {
            return InferredType::Reference(Box::new(InferredType::from_string(inner)));
        }

        // Handle generics
        if let Some(generic_start) = trimmed.find('<') {
            let name = trimmed[..generic_start].to_string();
            let args_str = &trimmed[generic_start + 1..trimmed.len() - 1];
            let args: Vec<InferredType> = args_str
                .split(',')
                .map(|s| InferredType::from_string(s.trim()))
                .collect();
            return InferredType::Generic(name, args);
        }

        // Handle Option/Result with type params
        if trimmed.starts_with("Option<") {
            let inner = &trimmed[7..trimmed.len() - 1];
            return InferredType::Generic(
                "Option".to_string(),
                vec![InferredType::from_string(inner)],
            );
        }
        if trimmed.starts_with("Result<") {
            let inner = &trimmed[7..trimmed.len() - 1];
            let parts: Vec<&str> = inner.split(',').collect();
            let mut args = Vec::new();
            for p in parts {
                args.push(InferredType::from_string(p.trim()));
            }
            return InferredType::Generic("Result".to_string(), args);
        }
        if trimmed.starts_with("Vec<") {
            let inner = &trimmed[4..trimmed.len() - 1];
            return InferredType::Generic(
                "Vec".to_string(),
                vec![InferredType::from_string(inner)],
            );
        }

        // Handle common types
        match trimmed {
            "String" | "str" | "&str" => InferredType::Concrete("String".to_string()),
            "bool" => InferredType::Concrete("bool".to_string()),
            "u8" | "u16" | "u32" | "u64" | "usize" => InferredType::Concrete("u64".to_string()),
            "i8" | "i16" | "i32" | "i64" | "isize" => InferredType::Concrete("i64".to_string()),
            "f32" | "f64" => InferredType::Concrete("f64".to_string()),
            "()" | "void" => InferredType::Concrete("()".to_string()),
            _ if trimmed.is_empty() => InferredType::Unknown,
            _ => InferredType::Concrete(trimmed.to_string()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TypeContext {
    pub variable_types: HashMap<String, InferredType>,
    pub function_return_types: HashMap<String, InferredType>,
    pub function_param_types: HashMap<String, Vec<InferredType>>,
    pub struct_fields: HashMap<String, HashMap<String, InferredType>>,
    pub type_aliases: HashMap<String, InferredType>,
}

impl TypeContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn infer_type(&self, expr: &str) -> InferredType {
        if let Some(t) = self.variable_types.get(expr) {
            return t.clone();
        }

        // Check if it's a type constructor
        if expr.chars().next().map_or(false, |c| c.is_uppercase()) {
            return InferredType::Concrete(expr.to_string());
        }

        // Check type aliases
        if let Some(t) = self.type_aliases.get(expr) {
            return t.clone();
        }

        InferredType::Unknown
    }

    pub fn register_struct(&mut self, name: &str, fields: HashMap<String, InferredType>) {
        self.struct_fields.insert(name.to_string(), fields);
    }

    pub fn register_function(
        &mut self,
        name: &str,
        params: Vec<InferredType>,
        return_type: InferredType,
    ) {
        self.function_param_types.insert(name.to_string(), params);
        self.function_return_types
            .insert(name.to_string(), return_type);
    }

    pub fn register_variable(
        &mut self,
        name: &str,
        type_hint: Option<&str>,
        initializer: Option<&str>,
    ) {
        self.register_variable_with_context(None, name, type_hint, initializer);
    }

    pub fn register_variable_with_context(
        &mut self,
        context: Option<&str>,
        name: &str,
        type_hint: Option<&str>,
        initializer: Option<&str>,
    ) {
        let key = context
            .map(|ctx| format!("{}::{}", ctx, name))
            .unwrap_or_else(|| name.to_string());

        if let Some(hint) = type_hint {
            self.variable_types
                .insert(key, InferredType::from_string(hint));
        } else if let Some(init) = initializer {
            let inferred = self.infer_from_initializer(init);
            self.variable_types.insert(key, inferred);
        }
    }

    pub fn register_type_alias(&mut self, alias: &str, target: InferredType) {
        self.type_aliases.insert(alias.to_string(), target);
    }

    pub fn lookup_field(&self, struct_name: &str, field: &str) -> Option<InferredType> {
        self.struct_fields
            .get(struct_name)
            .and_then(|fields| fields.get(field))
            .cloned()
    }

    pub fn lookup_function_return(&self, func_name: &str) -> Option<InferredType> {
        self.function_return_types.get(func_name).cloned()
    }

    pub fn infer_type_in_context(&self, context: Option<&str>, expr: &str) -> InferredType {
        if let Some(ctx) = context {
            let scoped_key = format!("{}::{}", ctx, expr);
            if let Some(t) = self.variable_types.get(&scoped_key) {
                return t.clone();
            }
        }

        self.infer_type(expr)
    }

    fn infer_from_initializer(&self, init: &str) -> InferredType {
        let trimmed = init.trim();

        // Type::new() or Type::default()
        if let Some(colon_pos) = trimmed.find("::") {
            let type_name = &trimmed[..colon_pos];
            if type_name.chars().next().map_or(false, |c| c.is_uppercase()) {
                return InferredType::Concrete(type_name.to_string());
            }
        }

        // String::from("...") or String::new()
        if trimmed.starts_with("String::") {
            return InferredType::Concrete("String".to_string());
        }

        // vec![...]
        if trimmed.starts_with("vec!") {
            return InferredType::Generic("Vec".to_string(), vec![InferredType::Unknown]);
        }

        // Literal strings
        if trimmed.starts_with('"') {
            return InferredType::Concrete("String".to_string());
        }

        // Numeric literals
        if trimmed.parse::<i64>().is_ok() {
            return InferredType::Concrete("i64".to_string());
        }
        if trimmed.parse::<f64>().is_ok() {
            return InferredType::Concrete("f64".to_string());
        }

        // Boolean literals
        if trimmed == "true" || trimmed == "false" {
            return InferredType::Concrete("bool".to_string());
        }

        // Method call: obj.method()
        if let Some(dot_pos) = trimmed.find('.') {
            let receiver = &trimmed[..dot_pos];
            if let Some(receiver_type) = self.variable_types.get(receiver) {
                return receiver_type.clone();
            }
        }

        // Function call: get_something()
        if trimmed.ends_with("()") {
            let func_name = trimmed.trim_end_matches("()");
            if let Some(ret) = self.function_return_types.get(func_name) {
                return ret.clone();
            }
        }

        InferredType::Unknown
    }
}
