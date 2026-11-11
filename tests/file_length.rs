use std::path::Path;

#[test]
fn source_files_stay_under_250_lines() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut too_long = Vec::new();
    for dir in ["src", "java-worker/src"] {
        walk(&root.join(dir), &mut too_long);
    }
    let build = root.join("build.rs");
    consider(&build, &mut too_long);
    assert!(too_long.is_empty(), "archivos sobre 250 líneas:\n{}", too_long.join("\n"));
}

fn walk(dir: &Path, too_long: &mut Vec<String>) {
    let entries = std::fs::read_dir(dir).unwrap();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, too_long);
        } else {
            consider(&path, too_long);
        }
    }
}

fn consider(path: &Path, too_long: &mut Vec<String>) {
    let ext = path.extension().and_then(|ext| ext.to_str());
    if !matches!(ext, Some("rs") | Some("java")) {
        return;
    }
    let text = std::fs::read_to_string(path).unwrap();
    let lines = text.lines().count();
    if lines > 250 {
        too_long.push(format!("{} ({lines})", path.display()));
    }
}
