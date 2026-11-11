use std::env;
use std::path::{Path, PathBuf};

/// Rutas de HEC-HMS 4.14. Un archivo nuevo de configuración solo toca este módulo.
#[derive(Debug, Clone)]
pub struct Config {
    pub hms_home: Option<PathBuf>,
    pub java_exe: Option<PathBuf>,
    pub hms_jar: Option<PathBuf>,
    pub lib_dir: Option<PathBuf>,
    pub bin_dir: Option<PathBuf>,
    pub hecdss_dll: Option<PathBuf>,
    pub worker_classes: PathBuf,
}

const DEFAULT_HOME: &str = r"C:\Program Files\HEC\HEC-HMS\4.14";

impl Config {
    pub fn resolve() -> Self {
        let home = env_path("HMS_HOME")
            .or_else(file_home)
            .or_else(default_home)
            .filter(|path| path.join("hms.jar").is_file());
        let hecdss = env_path("HECDSS_DLL")
            .filter(|path| path.is_file())
            .or_else(|| {
                home.as_ref()
                    .map(|root| root.join("bin").join("hecdss.dll"))
                    .filter(|path| path.is_file())
            })
            .or_else(bundled_hecdss);
        let worker = env_path("HMS_WORKER_CLASSES").unwrap_or_else(default_classes);
        Self {
            java_exe: home.as_ref().map(|root| root.join("jre").join("bin").join("java.exe")),
            hms_jar: home.as_ref().map(|root| root.join("hms.jar")),
            lib_dir: home.as_ref().map(|root| root.join("lib")),
            bin_dir: home.as_ref().map(|root| root.join("bin")),
            hms_home: home,
            hecdss_dll: hecdss,
            worker_classes: worker,
        }
    }
}

fn env_path(key: &str) -> Option<PathBuf> {
    env::var(key)
        .ok()
        .map(|value| PathBuf::from(value.trim()))
        .filter(|path| !path.as_os_str().is_empty())
}

fn file_home() -> Option<PathBuf> {
    let text = std::fs::read_to_string("hms.toml").ok()?;
    for line in text.lines() {
        let line = line.trim();
        let Some(value) = line.strip_prefix("hms_home") else {
            continue;
        };
        let value = value.trim().trim_start_matches('=').trim();
        let value = value.trim_matches('"').trim_matches('\'');
        if !value.is_empty() {
            return Some(PathBuf::from(value));
        }
    }
    None
}

fn default_home() -> Option<PathBuf> {
    let path = PathBuf::from(DEFAULT_HOME);
    path.is_dir().then_some(path)
}

fn default_classes() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("java-worker").join("classes")
}

fn bundled_hecdss() -> Option<PathBuf> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("native").join("hecdss.dll");
    path.is_file().then_some(path)
}
