//! Filters for dead code detection - prevents false positives

use crate::graph::call_graph::FunctionNode;

/// Check if a function should never be considered dead
pub fn is_never_dead(func: &FunctionNode) -> bool {
    // ⭐ NEW: Skip test functions (detected by parser)
    if func.is_test {
        return true;
    }

    // ⭐ NEW: Skip trait default methods
    if func.is_trait_default {
        return true;
    }

    // ⭐ NEW: Skip trait methods that are implemented
    if func.is_trait_method {
        return true;
    }

    // 1. Trait implementations (Rust, Go interfaces, etc.)
    if func.trait_impl.is_some() {
        return true;
    }

    // 2. Framework-decorated methods (detect by doc comments)
    if let Some(doc) = &func.doc_comment {
        // Common framework decorator patterns
        let decorator_patterns = [
            // HTTP method decorators
            "@Get",
            "@Post",
            "@Put",
            "@Delete",
            "@Patch",
            "@Options",
            "@Head",
            "@All",
            "@RequestMapping",
            "@RestController",
            "@Controller",
            "@Service",
            "@Repository",
            "@Component",
            "@Bean",
            "@Autowired",
            "@Qualifier",
            "@Value",
            // TypeScript/NestJS
            "@UseGuards",
            "@UseInterceptors",
            "@UsePipes",
            "@UseFilters",
            "@Injectable",
            "@Module",
            "@Global",
            "@Catch",
            // Python decorators
            "@app.route",
            "@app.get",
            "@app.post",
            "@app.put",
            "@app.delete",
            "@router.",
            "@blueprint.",
            "@login_required",
            "@permission_required",
            "@click.command",
            "@click.option",
            "@pytest",
            "@mock.patch",
            "@patch",
            "@staticmethod",
            "@classmethod",
            "@property",
            "@cached_property",
            "@dataclass",
            "@enum.unique",
            "@contextmanager",
            // Java annotations
            "@Override",
            "@Deprecated",
            "@SuppressWarnings",
            "@SafeVarargs",
            "@FunctionalInterface",
            "@Generated",
            "@Autowired",
            "@Qualifier",
            "@Value",
            "@Inject",
            "@Named",
            "@PostConstruct",
            "@PreDestroy",
            "@Transactional",
            "@Async",
            "@Scheduled",
            "@EventListener",
            "@ControllerAdvice",
            "@RestControllerAdvice",
            "@ExceptionHandler",
            "@InitBinder",
            "@ModelAttribute",
            "@SessionAttributes",
            "@Cacheable",
            "@CachePut",
            "@CacheEvict",
            "@CacheConfig",
            // Go //go: directives
            "//go:",
            "//export",
            "//cgo",
            // Rust attributes
            "#[derive",
            "#[cfg",
            "#[allow",
            "#[deny",
            "#[forbid",
            "#[macro_export]",
            "#[macro_use]",
            "#[proc_macro]",
            "#[test]",
            "#[bench]",
            "#[cfg(test)]",
            "#[async_trait]",
            "#[instrument]",
            "#[tracing::instrument]",
            "#[serde]",
            "#[serde::",
            "#[tokio::test]",
            "#[tokio::main]",
            "#[actix_web::",
            "#[rocket::",
            "#[axum::",
            "#[derive(Debug)]",
            "#[derive(Clone)]",
            "#[derive(Copy)]",
            "#[derive(PartialEq)]",
            "#[derive(Eq)]",
            "#[derive(Hash)]",
            "#[derive(Default)]",
            "#[derive(Serialize)]",
            "#[derive(Deserialize)]",
            "#[derive(From)]",
            "#[derive(Into)]",
            "#[derive(TryFrom)]",
            "#[derive(TryInto)]",
        ];

        for pattern in decorator_patterns {
            if doc.contains(pattern) {
                return true;
            }
        }

        // Also check for common doc patterns that indicate framework code
        if doc.contains("OpenAPI")
            || doc.contains("Swagger")
            || doc.contains("Schema")
            || doc.contains("Example")
        {
            return true;
        }
    }

    // 3. React/Vue/Svelte component props/hooks (destructured objects)
    if func.name.contains('{') && func.name.contains('}') {
        return true;
    }

    // 4. Standard trait/interface method names (language-agnostic)
    let common_trait_methods = [
        "fmt",
        "default",
        "from",
        "into",
        "try_from",
        "try_into",
        "clone",
        "drop",
        "as_ref",
        "as_mut",
        "borrow",
        "borrow_mut",
        "to_owned",
        "to_string",
        "into_iter",
        "iter",
        "iter_mut",
        "toString",
        "equals",
        "hashCode",
        "finalize",
        "clone_from",
        "partial_cmp",
        "eq",
        "ne",
        "lt",
        "le",
        "gt",
        "ge",
    ];
    if common_trait_methods.contains(&func.name.as_str()) {
        return true;
    }

    // 5. Entry points (language-agnostic)
    let entry_points = [
        "main",
        "async_main",
        "run",
        "start",
        "init",
        "setup",
        "main_async",
        "main_function",
        "entry",
    ];
    if entry_points.contains(&func.name.as_str()) {
        return true;
    }

    // 6. Check file path patterns for framework files
    let file = &func.file;
    let framework_file_patterns = [
        // Rust
        "/traits/",
        "/trait/",
        "/impls/",
        "/derive/",
        "/procedural/",
        "/macros/",
        "/macro/",
        "/generated/",
        "/gen/",
        "/protobuf/",
        "/pb/",
        // TypeScript/NestJS
        ".controller.",
        ".service.",
        ".module.",
        ".guard.",
        ".strategy.",
        ".interceptor.",
        ".pipe.",
        ".filter.",
        ".middleware.",
        ".decorator.",
        ".provider.",
        ".factory.",
        ".resolver.",
        ".directive.",
        ".plugin.",
        // React/Vue
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
        // Python
        "/admin/",
        "/management/",
        "/migrations/",
        "/serializers/",
        "/permissions/",
        "/throttling/",
        "/middleware/",
        "/signals/",
        "/validators/",
        // Java
        "/annotations/",
        "/enums/",
        "/constants/",
        "/aspects/",
        "/configurations/",
        "/properties/",
        "/repositories/",
        "/entities/",
        "/dtos/",
        "/mappers/",
        // Common
        "/tests/",
        "/test/",
        "/bench/",
        "/benches/",
        "/examples/",
        "/samples/",
        "/demo/",
        "/protos/",
        "/proto/",
        "/generated/",
        "/gen/",
        "/third_party/",
        "/vendor/",
        "/external/",
    ];

    for pattern in framework_file_patterns {
        if file.contains(pattern) {
            return true;
        }
    }

    false
}

/// Get the reason why a function is filtered
pub fn filter_reason(func: &FunctionNode) -> Option<&'static str> {
    // ⭐ NEW: Check test functions first
    if func.is_test {
        return Some("test_function");
    }

    // ⭐ NEW: Check trait default methods
    if func.is_trait_default {
        return Some("trait_default_method");
    }

    // ⭐ NEW: Check trait methods
    if func.is_trait_method {
        return Some("trait_method");
    }

    // Check trait implementations
    if func.trait_impl.is_some() {
        return Some("trait_implementation");
    }

    // Check doc comments for decorators
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
            "@app.route",
            "@router.",
            "@blueprint.",
            "#[derive",
            "#[cfg",
            "#[allow",
            "#[deny",
            "//go:",
            "//export",
        ];
        for pattern in decorator_patterns {
            if doc.contains(pattern) {
                return Some("framework_decorator");
            }
        }

        if doc.contains("OpenAPI") || doc.contains("Swagger") {
            return Some("openapi_doc");
        }
    }

    // React props
    if func.name.contains('{') && func.name.contains('}') {
        return Some("react_props");
    }

    // Common trait methods
    let common_trait_methods = [
        "fmt",
        "default",
        "from",
        "into",
        "try_from",
        "try_into",
        "clone",
        "drop",
        "as_ref",
        "as_mut",
        "borrow",
        "borrow_mut",
        "to_owned",
        "to_string",
        "into_iter",
        "iter",
        "iter_mut",
        "toString",
        "equals",
        "hashCode",
        "finalize",
    ];
    if common_trait_methods.contains(&func.name.as_str()) {
        return Some("trait_method");
    }

    // Entry points
    let entry_points = ["main", "async_main", "run", "start", "init"];
    if entry_points.contains(&func.name.as_str()) {
        return Some("entry_point");
    }

    // Framework file patterns
    let file = &func.file;
    let framework_file_patterns = [
        ".controller.",
        ".service.",
        ".module.",
        ".guard.",
        ".tsx",
        ".jsx",
        ".vue",
        ".svelte",
        "/components/",
        "/pages/",
        "/hooks/",
        "/tests/",
        "/test/",
        "/bench/",
        "/benches/",
        "/generated/",
        "/gen/",
        "/proto/",
        "/traits/",
        "/trait/",
        "/impls/",
        "/derive/",
        "/admin/",
        "/management/",
        "/migrations/",
        "/serializers/",
        "/permissions/",
        "/throttling/",
    ];
    for pattern in framework_file_patterns {
        if file.contains(pattern) {
            return Some("framework_file");
        }
    }

    None
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
