use std::path::PathBuf;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorData;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use super::result::pack;
use crate::server::HmsServer;

#[derive(Debug, Deserialize, JsonSchema)]
struct Args {
    /// Ruta del .hms o del .basin
    project: String,
    /// Nombre de la cuenca. Si se omite, usa la primera del proyecto.
    #[serde(default)]
    name: Option<String>,
}

#[tool_router(vis = "pub(crate)", router = basin_router)]
impl HmsServer {
    #[tool(description = "Elementos, área, aguas abajo y parámetros de una cuenca. No calcula.")]
    async fn hms_basin(&self, Parameters(args): Parameters<Args>) -> Result<String, ErrorData> {
        pack(self.projects.basin(&PathBuf::from(args.project), args.name.as_deref()))
    }
}
