// tests/fixtures/adversarial/mod.rs

use std::path::PathBuf;

pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/adversarial")
}

pub fn rust_fixtures() -> Vec<&'static str> {
    vec![
        "rust/trait_impl.rs",
        "rust/ffi_extern.rs",
        "rust/macro_used.rs",
        "rust/plugin_system.rs",
        "rust/generated_code.rs",
        "rust/dynamic_dispatch.rs",
    ]
}

pub fn python_fixtures() -> Vec<&'static str> {
    vec![
        "python/flask_route.py",
        "python/django_view.py",
        "python/plugin_entry.py",
        "python/dynamic_import.py",
        "python/decorator_chain.py",
    ]
}

pub fn typescript_fixtures() -> Vec<&'static str> {
    vec![
        "typescript/react_component.tsx",
        "typescript/nestjs_controller.ts",
        "typescript/decorator_usage.ts",
        "typescript/dynamic_import.ts",
    ]
}

pub fn go_fixtures() -> Vec<&'static str> {
    vec![
        "go/interface_impl.go",
        "go/init_function.go",
        "go/plugin_export.go",
        "go/cgo_ffi.go",
    ]
}

pub fn java_fixtures() -> Vec<&'static str> {
    vec![
        "java/spring_controller.java",
        "java/reflection_target.java",
        "java/service_loader.java",
        "java/annotation_processor.java",
    ]
}
