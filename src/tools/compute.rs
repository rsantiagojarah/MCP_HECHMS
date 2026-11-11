use std::path::PathBuf;
use std::time::Duration;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorData;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use super::result::pack;
use crate::server::HmsServer;

#[derive(Debug, Deserialize, JsonSchema)]
struct Args {
    /// Ruta del archivo .hms
    project: String,
    /// Nombre de la corrida, ensayo, pronóstico o análisis ya definido
    run: String,
    /// run, optimization, forecast, uncertainty, frequency, ensemble o depth-area. Por defecto run.
    #[serde(default)]
    kind: Option<String>,
    /// Segundos máximos de espera. Por defecto 3600.
    #[serde(default)]
    timeout_secs: Option<u64>,
}

#[tool_router(vis = "pub(crate)", router = compute_router)]
impl HmsServer {
    #[tool(description = "Calcula una corrida u otro análisis ya definido. kind: run, optimization, forecast, uncertainty, frequency, ensemble o depth-area. Cierra el DSS antes de simular.")]
    async fn hms_compute(&self, Parameters(args): Parameters<Args>) -> Result<String, ErrorData> {
        let project = PathBuf::from(&args.project);
        let kind = match crate::compute::canonical(args.kind.as_deref().unwrap_or("run")) {
            Ok(kind) => kind,
            Err(err) => return pack::<()>(Err(err)),
        };
        self.dss.release_all();
        let limit = Duration::from_secs(args.timeout_secs.unwrap_or(3600).clamp(1, 86_400));
        pack(self.compute.run(&project, &args.run, kind, limit).await)
    }
}
