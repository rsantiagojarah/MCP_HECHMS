use rmcp::model::ErrorData;
use rmcp::{tool, tool_router};
use serde::Serialize;

use super::result::ok_json;
use crate::server::HmsServer;

#[derive(Debug, Serialize)]
struct Ping {
    hms_home: Option<String>,
    java: Option<String>,
    hms_jar: Option<String>,
    hecdss_dll: Option<String>,
    worker_classes: String,
    note: &'static str,
}

#[tool_router(vis = "pub(crate)", router = ping_router)]
impl HmsServer {
    #[tool(description = "Estado de HEC-HMS 4.14, Java 21 y hecdss.dll. No abre proyectos ni calcula.")]
    async fn hms_ping(&self) -> Result<String, ErrorData> {
        let config = &self.config;
        ok_json(&Ping {
            hms_home: config.hms_home.as_ref().map(|path| path.display().to_string()),
            java: config.java_exe.as_ref().map(|path| path.display().to_string()),
            hms_jar: config.hms_jar.as_ref().map(|path| path.display().to_string()),
            hecdss_dll: config.hecdss_dll.as_ref().map(|path| path.display().to_string()),
            worker_classes: config.worker_classes.display().to_string(),
            note: "Las consultas de cuenca no usan Java. DSS 7 requiere hecdss.dll. El cómputo usa el worker Java.",
        })
    }
}
