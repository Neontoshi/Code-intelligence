// src/analysis/dead_code/filters.rs

use crate::graph::call_graph::FunctionNode;

/// ⭐ Protection levels for functions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProtectionLevel {
    /// Protected - never considered dead, safe from automatic deletion
    /// These are functions that are mathematically/semantically guaranteed to be alive
    Protected,

    /// Likely Alive - high confidence these are alive, but not mathematically guaranteed
    /// These include public API, entry points, etc.
    LikelyAlive,

    /// Candidate - may be dead, needs review
    /// These are the functions that should be analyzed by the verdict engine
    Candidate,
}

impl ProtectionLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProtectionLevel::Protected => "protected",
            ProtectionLevel::LikelyAlive => "likely_alive",
            ProtectionLevel::Candidate => "candidate",
        }
    }

    pub fn is_actionable(&self) -> bool {
        matches!(self, ProtectionLevel::Candidate)
    }

    pub fn is_safe_to_delete(&self) -> bool {
        matches!(self, ProtectionLevel::Candidate)
    }

    pub fn needs_review(&self) -> bool {
        matches!(
            self,
            ProtectionLevel::LikelyAlive | ProtectionLevel::Candidate
        )
    }
}

/// ⭐ Determine the protection level for a function
pub fn get_protection_level(func: &FunctionNode) -> ProtectionLevel {
    // 1. PROTECTED - Mathematically/semantically guaranteed alive

    // Test functions are protected
    if func.is_test {
        return ProtectionLevel::Protected;
    }

    // Trait default methods are protected
    if func.is_trait_default {
        return ProtectionLevel::Protected;
    }

    // Trait methods are protected
    if func.is_trait_method {
        return ProtectionLevel::Protected;
    }

    // Trait implementations are protected
    if func.trait_impl.is_some() {
        return ProtectionLevel::Protected;
    }

    // FFI functions are protected
    if let Some(doc) = &func.doc_comment {
        if doc.contains("extern \"C\"")
            || doc.contains("#[no_mangle]")
            || doc.contains("#[export_name]")
            || doc.contains("#[link_name]")
        {
            return ProtectionLevel::Protected;
        }
    }

    // 2. LIKELY ALIVE - High confidence but not guaranteed

    // Entry points are likely alive
    let entry_points = ["main", "async_main", "run", "start", "init", "setup"];
    if entry_points.contains(&func.name.as_str()) {
        return ProtectionLevel::LikelyAlive;
    }

    // Public API functions are likely alive (especially in libraries)
    if func.is_public && (func.file.contains("lib.rs") || func.file.contains("mod.rs")) {
        return ProtectionLevel::LikelyAlive;
    }

    // React components are likely alive
    if func.file.ends_with(".tsx") || func.file.ends_with(".jsx") {
        let is_component = func
            .name
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false);
        let is_hook = func.name.starts_with("use");
        if is_component || is_hook {
            return ProtectionLevel::LikelyAlive;
        }
    }

    // Framework-decorated functions are likely alive
    if let Some(doc) = &func.doc_comment {
        let decorator_patterns = [
            "@Get",
            "@Post",
            "@Put",
            "@Delete",
            "@Patch",
            "@RequestMapping",
            "@RestController",
            "@Controller",
            "@Service",
            "@Repository",
            "@Component",
            "@app.route",
            "@router.",
            "@blueprint.",
        ];
        for pattern in decorator_patterns {
            if doc.contains(pattern) {
                return ProtectionLevel::LikelyAlive;
            }
        }
    }

    // Functions with callers are likely alive
    if func.fan_in > 0 {
        return ProtectionLevel::LikelyAlive;
    }

    // Functions reachable from roots are likely alive
    // (This is checked by the verdict engine)

    // 3. CANDIDATE - May be dead, needs analysis
    ProtectionLevel::Candidate
}

/// Check if a function should never be considered dead (maintains backward compatibility)
pub fn is_never_dead(func: &FunctionNode) -> bool {
    matches!(get_protection_level(func), ProtectionLevel::Protected)
}

/// Check if a function is likely alive (not a good candidate for deletion)
pub fn is_likely_alive(func: &FunctionNode) -> bool {
    matches!(
        get_protection_level(func),
        ProtectionLevel::Protected | ProtectionLevel::LikelyAlive
    )
}

/// Get the reason why a function is filtered
pub fn filter_reason(func: &FunctionNode) -> Option<&'static str> {
    match get_protection_level(func) {
        ProtectionLevel::Protected => {
            if func.is_test {
                Some("protected: test_function")
            } else if func.is_trait_default {
                Some("protected: trait_default_method")
            } else if func.is_trait_method {
                Some("protected: trait_method")
            } else if func.trait_impl.is_some() {
                Some("protected: trait_implementation")
            } else {
                Some("protected: ffi_or_framework")
            }
        }
        ProtectionLevel::LikelyAlive => {
            if func.is_public {
                Some("likely_alive: public_api")
            } else if func.fan_in > 0 {
                Some("likely_alive: has_callers")
            } else {
                Some("likely_alive: entry_point")
            }
        }
        ProtectionLevel::Candidate => None,
    }
}

/// Check if a function is actionable (can be considered for deletion)
pub fn is_actionable(func: &FunctionNode) -> bool {
    get_protection_level(func).is_actionable()
}

/// Check if a file path suggests framework code (for bulk filtering)
pub fn is_framework_file(file: &str) -> bool {
    let framework_patterns = [
        ".controller.",
        ".service.",
        ".module.",
        ".guard.",
        ".strategy.",
        ".interceptor.",
        ".pipe.",
        ".filter.",
        ".tsx",
        ".jsx",
        ".vue",
        ".svelte",
        "/components/",
        "/pages/",
        "/hooks/",
        "/composables/",
        "/providers/",
        "/contexts/",
        "/layouts/",
        "/tests/",
        "/test/",
        "/bench/",
        "/benches/",
        "/generated/",
        "/gen/",
        "/proto/",
        "/protobuf/",
        "/admin/",
        "/management/",
        "/migrations/",
        "/serializers/",
        "/permissions/",
        "/throttling/",
        "/traits/",
        "/trait/",
        "/impls/",
        "/derive/",
    ];
    for pattern in framework_patterns {
        if file.contains(pattern) {
            return true;
        }
    }
    false
}

///  Get a human-readable description of the protection level
pub fn protection_level_description(level: ProtectionLevel) -> &'static str {
    match level {
        ProtectionLevel::Protected => {
            "Protected - Never considered dead (trait impl, FFI, test, etc.)"
        }
        ProtectionLevel::LikelyAlive => {
            "Likely Alive - High confidence alive (public API, entry point, has callers)"
        }
        ProtectionLevel::Candidate => "Candidate - May be dead, needs analysis",
    }
}

///  Get the protection level as a color/emoji for display
pub fn protection_level_emoji(level: ProtectionLevel) -> &'static str {
    match level {
        ProtectionLevel::Protected => "🛡️",
        ProtectionLevel::LikelyAlive => "🟢",
        ProtectionLevel::Candidate => "🟠",
    }
}

///  Check if a function should be prioritized for review
pub fn should_prioritize_review(func: &FunctionNode) -> bool {
    let level = get_protection_level(func);
    match level {
        ProtectionLevel::Candidate => func.is_public || func.complexity > 10.0,
        _ => false,
    }
}
