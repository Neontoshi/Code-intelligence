// src/analysis/dead_code/filters/csharp.rs

//! C#-specific dead code filters

use super::common::is_test_file;
use super::{LanguageFilter, ProtectionLevel};
use crate::graph::call_graph::FunctionNode;

pub struct CSharpFilter;

impl LanguageFilter for CSharpFilter {
    fn get_protection_level(&self, func: &FunctionNode) -> ProtectionLevel {
        // 1. PROTECTED

        // Test functions are protected
        if func.is_test || is_test_file(&func.file) {
            return ProtectionLevel::Protected;
        }

        // .NET lifecycle methods
        if func.file.ends_with(".cs") {
            let dotnet_lifecycle = [
                "Main",
                "ConfigureServices",
                "Configure",
                "OnInitialized",
                "OnInitializedAsync",
                "OnParametersSet",
                "OnParametersSetAsync",
                "Dispose",
                "DisposeAsync",
                "ToString",
                "GetHashCode",
                "Equals",
            ];
            if dotnet_lifecycle.contains(&func.name.as_str()) {
                return ProtectionLevel::Protected;
            }
        }

        // 2. LIKELY ALIVE

        // ASP.NET Core action verbs
        let action_prefixes = ["Get", "Post", "Put", "Delete", "Patch", "OnGet", "OnPost"];
        if action_prefixes.iter().any(|p| func.name.starts_with(p)) && func.is_public {
            return ProtectionLevel::LikelyAlive;
        }

        // Functions with callers are likely alive
        if func.fan_in > 0 {
            return ProtectionLevel::LikelyAlive;
        }

        // 3. CANDIDATE
        ProtectionLevel::Candidate
    }
}
