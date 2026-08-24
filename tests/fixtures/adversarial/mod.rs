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
    ]
}

pub fn python_fixtures() -> Vec<&'static str> {
    vec!["python/flask_route.py", "python/dynamic_import.py"]
}

pub fn typescript_fixtures() -> Vec<&'static str> {
    vec![
        "typescript/react_component.tsx",
        "typescript/nextjs_controller.ts",
    ]
}

pub fn go_fixtures() -> Vec<&'static str> {
    vec!["go/interface_impl.go", "go/plugin_export.go"]
}

pub fn java_fixtures() -> Vec<&'static str> {
    vec!["java/spring_controller.java"]
}

pub fn csharp_fixtures() -> Vec<&'static str> {
    vec!["csharp/aspnet_controller.cs"]
}

pub fn dart_fixtures() -> Vec<&'static str> {
    vec!["dart/flutter_widget.dart"]
}

pub fn php_fixtures() -> Vec<&'static str> {
    vec!["php/laravel_controller.php"]
}

pub fn cpp_fixtures() -> Vec<&'static str> {
    vec!["cpp/virtual_member.cpp"]
}
