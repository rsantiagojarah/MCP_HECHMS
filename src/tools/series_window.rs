use std::path::PathBuf;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorData;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use super::result::pack;
use crate::dss::TimeWindow;
use crate::server::HmsServer;

#[derive(Debug, Deserialize, JsonSchema)]
struct Args {
    /// Ruta del archivo DSS 7
    dss: String,
    /// Pathname completo de la serie
    pathname: String,
    /// Fecha inicial HEC, por ejemplo 01Jan2020
    start_date: String,
    /// Fecha final HEC, por ejemplo 03Jan2020
    end_date: String,
    #[serde(default)]
    start_time: Option<String>,
    #[serde(default)]
    end_time: Option<String>,
    /// Puntos devueltos. Tope 500.
    #[serde(default)]
    max_points: Option<u32>,
}

#[tool_router(vis = "pub(crate)", router = series_window_router)]
impl HmsServer {
    #[tool(description = "Ventana corta de una serie DSS 7. Como máximo 500 puntos, con muestreo si hace falta.")]
    async fn hms_series_window(&self, Parameters(args): Parameters<Args>) -> Result<String, ErrorData> {
        let window = TimeWindow::resolve(
            &args.pathname,
            &args.start_date,
            args.start_time.as_deref().unwrap_or(""),
            &args.end_date,
            args.end_time.as_deref().unwrap_or(""),
        );
        let max_points = args.max_points.unwrap_or(500) as usize;
        pack(self.dss.window(&PathBuf::from(args.dss), &args.pathname, &window, max_points))
    }
}
