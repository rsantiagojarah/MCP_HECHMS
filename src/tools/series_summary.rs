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
    #[serde(default)]
    start_date: Option<String>,
    #[serde(default)]
    start_time: Option<String>,
    #[serde(default)]
    end_date: Option<String>,
    #[serde(default)]
    end_time: Option<String>,
}

#[tool_router(vis = "pub(crate)", router = series_summary_router)]
impl HmsServer {
    #[tool(description = "Pico, tiempo al pico, mínimo e integral de una serie DSS 7. No devuelve el hidrograma.")]
    async fn hms_series_summary(&self, Parameters(args): Parameters<Args>) -> Result<String, ErrorData> {
        let window = window_of(&args);
        pack(self.dss.summary(&PathBuf::from(args.dss), &args.pathname, &window))
    }
}

fn window_of(args: &Args) -> TimeWindow {
    TimeWindow::resolve(
        &args.pathname,
        args.start_date.as_deref().unwrap_or(""),
        args.start_time.as_deref().unwrap_or(""),
        args.end_date.as_deref().unwrap_or(""),
        args.end_time.as_deref().unwrap_or(""),
    )
}
