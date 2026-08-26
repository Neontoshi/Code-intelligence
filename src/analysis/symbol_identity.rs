use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SymbolIdentity {
    pub repository: String,
    pub module: String,
    pub file: String,
    pub language: String,
    pub qualified_symbol: String,
    pub signature: String,
    pub body_hash: String,
}

impl SymbolIdentity {
    pub fn new(
        repository: &str,
        module: &str,
        file: &str,
        language: &str,
        qualified_symbol: &str,
        signature: &str,
        body_hash: &str,
    ) -> Self {
        Self {
            repository: repository.to_string(),
            module: module.to_string(),
            file: file.to_string(),
            language: language.to_string(),
            qualified_symbol: qualified_symbol.to_string(),
            signature: signature.to_string(),
            body_hash: body_hash.to_string(),
        }
    }

    pub fn stable_id(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.repository.as_bytes());
        hasher.update(b"::");
        hasher.update(self.module.as_bytes());
        hasher.update(b"::");
        hasher.update(self.qualified_symbol.as_bytes());
        hasher.update(b"::");
        hasher.update(self.signature.as_bytes());
        let hash = hex::encode(hasher.finalize());
        format!("sym_{}", &hash[..20])
    }
}
