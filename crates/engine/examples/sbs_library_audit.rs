//! Inspect application import coverage separately from the native SBS conversion checks.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let report = std::env::args().nth(1).ok_or("usage: sbs_library_audit <report.json>")?;
    let start = std::time::Instant::now();
    let files = materialcraft_formats::sbs::library::files();
    let library = materialcraft_engine::NodeLibrary::load(&files);
    let mut graphs = 0;
    let mut functions = 0;
    let mut function_errors = Vec::new();
    let mut resources = 0;
    let mut missing_resources = Vec::new();
    for (path, package) in &library.packages {
        for resource in &package.resources {
            resources += 1;
            if resource.bytes.is_empty() {
                missing_resources.push(format!("{path}#{}", resource.sbs_path()));
            }
        }
        for graph in &package.graphs {
            graphs += 1;
            if let Some(function) = &graph.function {
                functions += 1;
                let mut resolved = function.clone();
                materialcraft_nodes::function::resolve_instances(&mut resolved, package, &library.packages);
                if let Err(error) = materialcraft_nodes::function::compile(&resolved, 0) {
                    function_errors.push(serde_json::json!({"package": path, "graph": graph.sbs_path(), "error": error}));
                }
            }
        }
    }
    let result = serde_json::json!({
        "files": files.len(), "loaded_packages": library.packages.len(), "graphs": graphs,
        "standalone_functions": functions, "resources": resources, "resource_declarations_without_bytes": missing_resources,
        "import_warnings": library.warnings, "function_compile_errors": function_errors,
        "elapsed_seconds": start.elapsed().as_secs_f64(), "external_runtime_validated": false,
    });
    materialcraft_formats::write_atomic(std::path::Path::new(&report), &serde_json::to_vec_pretty(&result)?)?;
    eprintln!(
        "{} packages, {graphs} graphs, {} import warnings, {} function compile errors, {} resource declarations without bytes ({:.2}s).",
        library.packages.len(),
        library.warnings.len(),
        function_errors.len(),
        missing_resources.len(),
        start.elapsed().as_secs_f64()
    );
    Ok(())
}
