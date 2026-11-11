use std::path::PathBuf;
use std::process::Stdio;

use tokio::process::Command;

use crate::config::Config;
use crate::error::HmsError;

pub struct JavaLaunch {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub env_path: String,
    pub proj_lib: PathBuf,
    pub gdal_data: PathBuf,
}

pub fn launch(config: &Config) -> Result<JavaLaunch, HmsError> {
    let home = config
        .hms_home
        .clone()
        .ok_or_else(|| HmsError::message("no está HEC-HMS 4.14. Defina HMS_HOME"))?;
    let program = config
        .java_exe
        .clone()
        .filter(|path| path.is_file())
        .ok_or_else(|| HmsError::message("no está java.exe dentro de HEC-HMS 4.14"))?;
    let jar = config
        .hms_jar
        .clone()
        .filter(|path| path.is_file())
        .ok_or_else(|| HmsError::message("no está hms.jar"))?;
    let classes = &config.worker_classes;
    let main = classes.join("hmsmcp").join("Main.class");
    if !main.is_file() {
        return Err(HmsError::message(
            "el worker Java no está compilado. Instale un JDK 21 y ejecute cargo build",
        ));
    }
    let lib = home.join("lib");
    let bin = home.join("bin");
    let classpath = format!("{};{};{}\\*", classes.display(), jar.display(), lib.display());
    let jars = format!("{};{}\\*", jar.display(), lib.display());
    let native = format!("{};{};{}", bin.display(), bin.join("gdal").display(), bin.join("hdf").display());
    let path = format!(
        "{};{};{};{};{}",
        bin.display(),
        bin.join("gdal").display(),
        bin.join("taudem").display(),
        bin.join("netcdf").display(),
        std::env::var("PATH").unwrap_or_default()
    );
    Ok(JavaLaunch {
        program,
        args: vec![
            "-Djava.awt.headless=true".into(),
            "-Dsun.java2d.d3d=false".into(),
            "-DMapPanel.NoVolatileImage=true".into(),
            "-Dpython.import.site=false".into(),
            format!("-Dpython.path={jars}"),
            "-Djavax.net.ssl.trustStoreType=Windows-ROOT".into(),
            "-Djavax.net.ssl.trustStore=NONE".into(),
            "-Xms32M".into(),
            format!("-Djava.library.path={native}"),
            "-classpath".into(),
            classpath,
            "hmsmcp.Main".into(),
        ],
        env_path: path,
        proj_lib: bin.join("gdal").join("projlib"),
        gdal_data: bin.join("gdal").join("gdal-data"),
    })
}

pub fn command(spec: &JavaLaunch) -> Command {
    let mut cmd = Command::new(&spec.program);
    cmd.args(&spec.args)
        .env("PATH", &spec.env_path)
        .env("PROJ_LIB", &spec.proj_lib)
        .env("GDAL_DATA", &spec.gdal_data)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    cmd
}
