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
    /// Ruta del archivo .hms
    project: String,
}

#[tool_router(vis = "pub(crate)", router = project_info_router)]
impl HmsServer {
    #[tool(description = "Lista cuencas, meteorología, controles y corridas de un proyecto HEC-HMS. No calcula.")]
    async fn hms_project_info(&self, Parameters(args): Parameters<Args>) -> Result<String, ErrorData> {
        pack(self.projects.info(&PathBuf::from(args.project)))
    }
}
