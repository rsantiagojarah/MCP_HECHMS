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
    /// basin, control, met o gage. Por defecto basin.
    #[serde(default)]
    target: Option<String>,
    /// Nombre de la cuenca, el control o la meteorología, si hay más de uno
    #[serde(default)]
    basin: Option<String>,
    /// Nombre del elemento, por ejemplo Subbasin-1
    element: String,
    /// Parámetro tal como aparece en el archivo, por ejemplo Lag
    parameter: String,
    /// Valor nuevo, en una sola línea
    value: String,
}

#[tool_router(vis = "pub(crate)", router = set_parameter_router)]
impl HmsServer {
    #[tool(description = "Cambia un parámetro de la cuenca, el control, la meteorología o un pluviómetro y lo guarda. No calcula.")]
    async fn hms_set_parameter(&self, Parameters(args): Parameters<Args>) -> Result<String, ErrorData> {
        let path = PathBuf::from(&args.project);
        self.compute.invalidate(&path).await;
        let changed = self.projects.set_in(
            &path,
            args.target.as_deref().unwrap_or("basin"),
            args.basin.as_deref(),
            &args.element,
            &args.parameter,
            &args.value,
        );
        pack(changed)
    }
}
