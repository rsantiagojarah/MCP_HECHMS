use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=java-worker/src");
    let manifest = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).to_path_buf();
    let src = manifest.join("java-worker").join("src");
    let out = manifest.join("java-worker").join("classes");
    let Some(javac) = find_javac() else {
        println!("cargo:warning=javac no está en PATH. El worker de HEC-HMS no se compiló.");
        return;
    };
    let files = ["Main.java", "Engine.java", "Protocol.java", "Reply.java"];
    let mut command = Command::new(javac);
    command.arg("-encoding").arg("UTF-8").arg("-d").arg(&out);
    for file in files {
        command.arg(src.join("hmsmcp").join(file));
    }
    if std::fs::create_dir_all(&out).is_err() {
        println!("cargo:warning=no se pudo crear java-worker/classes");
        return;
    }
    match command.status() {
        Ok(status) if status.success() => {}
        Ok(status) => println!("cargo:warning=javac terminó con {status}"),
        Err(err) => println!("cargo:warning=no se ejecutó javac: {err}"),
    }
}

fn find_javac() -> Option<std::path::PathBuf> {
    println!("cargo:rerun-if-env-changed=JAVA_HOME");
    println!("cargo:rerun-if-env-changed=LOCALAPPDATA");
    javac_under(std::env::var_os("JAVA_HOME"))
        .or_else(javac_on_path)
        .or_else(portable_javac)
}

fn javac_under(home: Option<std::ffi::OsString>) -> Option<std::path::PathBuf> {
    let home = home?;
    let candidate = Path::new(&home).join("bin").join("javac.exe");
    candidate.is_file().then_some(candidate)
}

fn javac_on_path() -> Option<std::path::PathBuf> {
    let ok = Command::new("javac").arg("-version").status().is_ok_and(|s| s.success());
    ok.then(|| std::path::PathBuf::from("javac"))
}

fn portable_javac() -> Option<std::path::PathBuf> {
    let root = std::env::var_os("LOCALAPPDATA")?;
    let entries = std::fs::read_dir(Path::new(&root).join("mcp-hechms-jdk")).ok()?;
    entries.flatten().find_map(|entry| {
        let candidate = entry.path().join("bin").join("javac.exe");
        candidate.is_file().then_some(candidate)
    })
}
