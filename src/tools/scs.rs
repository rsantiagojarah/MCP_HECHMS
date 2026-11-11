use std::path::PathBuf;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorData;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use super::result::pack;
use crate::project::ScsInput;
use crate::server::HmsServer;

#[derive(Debug, Deserialize, JsonSchema)]
struct Args {
    /// Ruta del archivo .hms o del .basin
    project: String,
    /// Nombre de la cuenca, si el proyecto tiene más de una
    #[serde(default)]
    basin: Option<String>,
    /// Subcuenca, por ejemplo Subbasin-1
    element: String,
    /// Número de curva del SCS, entre 1 y 100
    curve_number: f64,
    /// Tiempo de retardo del hidrograma unitario, en minutos
    lag_minutes: f64,
    /// Abstracción inicial. Si se omite, se usa el 20 % de la retención potencial
    #[serde(default)]
    initial_abstraction: Option<f64>,
    /// Porcentaje impermeable, entre 0 y 100
    #[serde(default)]
    impervious_percent: Option<f64>,
    /// standard o delmarva. Por defecto standard
    #[serde(default)]
    unitgraph: Option<String>,
}

#[tool_router(vis = "pub(crate)", router = scs_router)]
impl HmsServer {
    #[tool(description = "Configura en una subcuenca el número de curva del SCS para la precipitación efectiva y el hidrograma unitario del SCS. El retardo se da en minutos.")]
    async fn hms_apply_scs(&self, Parameters(args): Parameters<Args>) -> Result<String, ErrorData> {
        let path = PathBuf::from(&args.project);
        self.compute.invalidate(&path).await;
        let applied = self.projects.apply_scs(
            &path,
            args.basin.as_deref(),
            ScsInput {
                element: &args.element,
                curve_number: args.curve_number,
                lag_minutes: args.lag_minutes,
                initial_abstraction: args.initial_abstraction,
                impervious_percent: args.impervious_percent,
                unitgraph: args.unitgraph.as_deref(),
            },
        );
        pack(applied)
    }
}
