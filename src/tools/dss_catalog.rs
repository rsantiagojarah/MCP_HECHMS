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
    /// Ruta del archivo DSS 7
    dss: String,
    /// Filtro de pathname, por ejemplo /*/*/*FLOW*/*/*/*/
    #[serde(default)]
    filter: Option<String>,
    /// Máximo de pathnames devueltos. Tope 1000.
    #[serde(default)]
    limit: Option<u32>,
}

#[tool_router(vis = "pub(crate)", router = dss_catalog_router)]
impl HmsServer {
    #[tool(description = "Catálogo de un DSS 7. No calcula y no devuelve los valores de las series.")]
    async fn hms_dss_catalog(&self, Parameters(args): Parameters<Args>) -> Result<String, ErrorData> {
        let limit = args.limit.unwrap_or(100).clamp(1, 1000) as usize;
        let filter = args.filter.as_deref().unwrap_or("/*/*/*/*/*/*/").to_string();
        pack(self.dss.catalog(&PathBuf::from(args.dss), &filter, limit))
    }
}
