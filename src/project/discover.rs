use std::path::{Path, PathBuf};

pub fn sibling(project: &Path, filename: &str) -> PathBuf {
    let parent = project.parent().unwrap_or(Path::new("."));
    parent.join(filename)
}
