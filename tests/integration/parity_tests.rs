use code_intelligence::parser::tree_sitter::TreeSitterParser;
use std::path::PathBuf;

#[test]
fn test_cross_language_parity() {
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parity");

    if !fixtures_dir.exists() {
        eprintln!("Skipping parity tests - fixtures directory not found");
        return;
    }

    let languages = [
        "rust",
        "python",
        "javascript",
        "typescript",
        "go",
        "java",
        "dart",
        "php",
        "cpp",
        "csharp",
    ];
    let patterns = [
        "obviously_dead",
        "public_api",
        "dynamic_ref",
        "framework_entry",
        "recursive",
    ];

    let parser = TreeSitterParser::new();

    for pattern in patterns {
        println!("\n=== Testing pattern: {} ===", pattern);

        for language in languages {
            let file_name = match (pattern, language) {
                ("framework_entry", "javascript") => "framework_entry.jsx",
                ("framework_entry", "typescript") => "framework_entry.tsx",
                _ => match pattern {
                    "obviously_dead" => match language {
                        "rust" => "obviously_dead.rs",
                        "python" => "obviously_dead.py",
                        "javascript" => "obviously_dead.js",
                        "typescript" => "obviously_dead.ts",
                        "go" => "obviously_dead.go",
                        "java" => "obviously_dead.java",
                        "dart" => "obviously_dead.dart",
                        "php" => "obviously_dead.php",
                        "cpp" => "obviously_dead.cpp",
                        "csharp" => "obviously_dead.cs",
                        _ => continue,
                    },
                    "public_api" => match language {
                        "rust" => "public_api.rs",
                        "python" => "public_api.py",
                        "javascript" => "public_api.js",
                        "typescript" => "public_api.ts",
                        "go" => "public_api.go",
                        "java" => "public_api.java",
                        "dart" => "public_api.dart",
                        "php" => "public_api.php",
                        "cpp" => "public_api.cpp",
                        "csharp" => "public_api.cs",
                        _ => continue,
                    },
                    "dynamic_ref" => match language {
                        "rust" => "dynamic_ref.rs",
                        "python" => "dynamic_ref.py",
                        "javascript" => "dynamic_ref.js",
                        "typescript" => "dynamic_ref.ts",
                        "go" => "dynamic_ref.go",
                        "java" => "dynamic_ref.java",
                        "dart" => "dynamic_ref.dart",
                        "php" => "dynamic_ref.php",
                        "cpp" => "dynamic_ref.cpp",
                        "csharp" => "dynamic_ref.cs",
                        _ => continue,
                    },
                    "framework_entry" => match language {
                        "rust" => "framework_entry.rs",
                        "python" => "framework_entry.py",
                        "javascript" => "framework_entry.jsx",
                        "typescript" => "framework_entry.tsx",
                        "go" => "framework_entry.go",
                        "java" => "framework_entry.java",
                        "dart" => "framework_entry.dart",
                        "php" => "framework_entry.php",
                        "cpp" => "framework_entry.cpp",
                        "csharp" => "framework_entry.cs",
                        _ => continue,
                    },
                    "recursive" => match language {
                        "rust" => "recursive.rs",
                        "python" => "recursive.py",
                        "javascript" => "recursive.js",
                        "typescript" => "recursive.ts",
                        "go" => "recursive.go",
                        "java" => "recursive.java",
                        "dart" => "recursive.dart",
                        "php" => "recursive.php",
                        "cpp" => "recursive.cpp",
                        "csharp" => "recursive.cs",
                        _ => continue,
                    },
                    _ => continue,
                },
            };

            let full_path = fixtures_dir.join(language).join(file_name);
            if !full_path.exists() {
                eprintln!("  {}: MISSING fixture", language);
                continue;
            }

            match parser.parse_file(&full_path) {
                Ok(_) => {
                    println!("  {}: PARSED successfully", language);
                }
                Err(e) => {
                    println!("  {}: PARSE ERROR: {:?}", language, e);
                }
            }
        }
    }
}
