use crate::graph::call_graph::FunctionNode;

pub fn is_never_dead(func: &FunctionNode) -> bool {
    // Skip test functions (detected by parser)
    if func.is_test {
        return true;
    }

    // Skip trait default methods
    if func.is_trait_default {
        return true;
    }

    // Skip trait methods that are implemented
    if func.is_trait_method {
        return true;
    }

    // 1. Trait implementations (Rust, Go interfaces, etc.)
    if func.trait_impl.is_some() {
        return true;
    }

    // ⭐ NEW: Check if function is in a file that contains trait implementations
    // This catches cases where the parser didn't capture the trait relationship
    if func.file.contains("trait_impl")
        || func.file.contains("impls")
        || func.file.contains("traits")
    {
        // If it's a common trait method name, it's almost certainly a trait impl
        let common_trait_methods = [
            "handle",
            "process",
            "execute",
            "run",
            "call",
            "invoke",
            "get",
            "set",
            "is",
            "has",
            "with",
            "without",
            "fmt",
            "default",
            "from",
            "into",
            "clone",
            "drop",
            "as_ref",
            "as_mut",
            "borrow",
            "to_string",
            "into_iter",
            "iter",
            "iter_mut",
            "toString",
            "equals",
            "hashCode",
        ];
        if common_trait_methods.contains(&func.name.as_str()) {
            return true;
        }
    }

    // ⭐ NEW: Check if function name strongly suggests trait implementation
    let strong_trait_names = [
        "handle", "process", "execute", "run", "call", "invoke", "clone", "drop", "default",
        "from", "into",
    ];
    if strong_trait_names.contains(&func.name.as_str()) {
        // If it's in a file with any trait-like path
        if func.file.contains("trait")
            || func.file.contains("impl")
            || func.file.contains("interface")
        {
            return true;
        }
    }

    // 2. React components (TSX/JSX)
    if func.file.ends_with(".tsx") || func.file.ends_with(".jsx") {
        let is_component = func
            .name
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false);
        let is_hook = func.name.starts_with("use");
        let is_default_export = func
            .doc_comment
            .as_ref()
            .map(|d| d.contains("export default"))
            .unwrap_or(false);

        if is_component || is_hook || is_default_export {
            return true;
        }
    }

    // 3. React patterns in doc comment
    if let Some(doc) = &func.doc_comment {
        if doc.contains("React.FC")
            || doc.contains("React.Component")
            || doc.contains("React.memo")
            || doc.contains("React.forwardRef")
            || doc.contains("useState")
            || doc.contains("useEffect")
            || doc.contains("useContext")
            || doc.contains("useReducer")
        {
            return true;
        }
    }

    // 4. Framework-decorated methods (detect by doc comments)
    if let Some(doc) = &func.doc_comment {
        let decorator_patterns = [
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
            "@UseGuards",
            "@UseInterceptors",
            "@UsePipes",
            "@UseFilters",
            "@Injectable",
            "@Module",
            "@Global",
            "@Catch",
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
            "//go:",
            "//export",
            "//cgo",
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

        if doc.contains("OpenAPI") || doc.contains("Swagger") || doc.contains("Schema") {
            return true;
        }
    }

    // 5. React/Vue/Svelte component props/hooks
    if func.name.contains('{') && func.name.contains('}') {
        return true;
    }

    // 6. Standard trait/interface method names (language-agnostic)
    let trait_methods = [
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
    if trait_methods.contains(&func.name.as_str()) {
        return true;
    }

    // 7. Entry points
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

    // 8. Check file path patterns for framework files
    let file = &func.file;
    let framework_file_patterns = [
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
        "/admin/",
        "/management/",
        "/migrations/",
        "/serializers/",
        "/permissions/",
        "/throttling/",
        "/middleware/",
        "/signals/",
        "/validators/",
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
        "trait_impl",
        "trait_",
        "_impl",
        "impl_",
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
