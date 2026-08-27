// benches/stress_benchmark.rs

use code_intelligence::graph::GraphMetrics;
use code_intelligence::Pipeline;
use std::time::Instant;

fn generate_massive_repo(size: usize) -> tempfile::TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path();

    let files_needed = (size / 100).max(1);

    for file_idx in 0..files_needed {
        let mut code = String::new();
        code.push_str("use std::collections::HashMap;\n\n");

        for func_idx in 0usize..100 {
            code.push_str(&format!(
                "pub fn func_{}_{}(x: i32) -> i32 {{\n",
                file_idx, func_idx
            ));
            if func_idx % 3 == 0 {
                code.push_str(&format!(
                    "    func_{}_{}(x) * 2\n",
                    file_idx,
                    func_idx.saturating_sub(1)
                ));
            } else {
                code.push_str("    x + 1\n");
            }
            code.push_str("}\n\n");
        }

        std::fs::write(path.join(format!("file_{:06}.rs", file_idx)), code).unwrap();
    }

    std::fs::write(path.join("main.rs"), "pub fn main() { func_0_0(42); }").unwrap();
    std::fs::write(
        path.join("Cargo.toml"),
        "[package]\nname = \"stress\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();

    temp_dir
}

fn main() {
    let sizes = [50000, 100000, 500000];

    for &size in &sizes {
        println!("\n🔄 Generating {} functions...", size);
        let temp_dir = generate_massive_repo(size);
        let path = temp_dir.path();

        println!("⚙️ Analyzing...");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let start = Instant::now();
        let mut pipeline = Pipeline::new();
        let analysis = rt.block_on(pipeline.process_project(path)).unwrap();
        let elapsed = start.elapsed();

        println!("📊 Results for {} functions:", size);
        println!("   Actual functions: {}", analysis.call_graph.node_count());
        println!("   Edges: {}", analysis.call_graph.edge_count());
        println!("   Files: {}", analysis.files.len());
        println!("   Time: {:.2}s", elapsed.as_secs_f64());
        println!(
            "   Functions/s: {:.0}",
            analysis.call_graph.node_count() as f64 / elapsed.as_secs_f64()
        );
        println!(
            "   Files/s: {:.0}",
            analysis.files.len() as f64 / elapsed.as_secs_f64()
        );
    }
}
