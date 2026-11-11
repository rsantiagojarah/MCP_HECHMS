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
    /// Ruta del DSS 7. Si no existe, se crea.
    dss: String,
    /// Pathname completo, con el intervalo en la parte E. Ejemplo: /A/B/FLOW/01Jan2020/1Hour/MCP/
    pathname: String,
    /// Fecha inicial, por ejemplo 01Jan2020
    start_date: String,
    /// Hora inicial HEC, por ejemplo 0000. Por defecto 0000.
    #[serde(default)]
    start_time: Option<String>,
    /// Valores de la serie regular. Entre 1 y 20000.
    values: Vec<f64>,
    /// Unidades. Por defecto M3/S.
    #[serde(default)]
    units: Option<String>,
    /// Tipo DSS: INST-VAL, PER-AVER, PER-CUM o INST-CUM. Por defecto INST-VAL.
    #[serde(default)]
    data_type: Option<String>,
}

#[tool_router(vis = "pub(crate)", router = dss_store_router)]
impl HmsServer {
    #[tool(description = "Escribe una serie regular en un DSS 7. El intervalo sale de la parte E del pathname.")]
    async fn hms_dss_store_series(&self, Parameters(args): Parameters<Args>) -> Result<String, ErrorData> {
        let start = args.start_time.unwrap_or_else(|| "0000".into());
        let units = args.units.unwrap_or_else(|| "M3/S".into());
        let data_type = args.data_type.unwrap_or_else(|| "INST-VAL".into());
        pack(self.dss.store_series(
            &PathBuf::from(args.dss),
            &args.pathname,
            &args.start_date,
            &start,
            &args.values,
            &units,
            &data_type,
        ))
    }

    #[tool(description = "Escribe una serie irregular. times son minutos desde start_date, uno por cada valor.")]
    async fn hms_dss_store_irregular(&self, Parameters(args): Parameters<IrregularArgs>) -> Result<String, ErrorData> {
        let units = args.units.unwrap_or_else(|| "M3/S".into());
        let data_type = args.data_type.unwrap_or_else(|| "INST-VAL".into());
        pack(self.dss.store_irregular(
            &PathBuf::from(args.dss),
            &args.pathname,
            &args.start_date,
            &args.times,
            &args.values,
            &units,
            &data_type,
        ))
    }

    #[tool(description = "Borra un registro de un DSS 7.")]
    async fn hms_dss_delete(&self, Parameters(args): Parameters<DeleteArgs>) -> Result<String, ErrorData> {
        pack(self.dss.delete(&PathBuf::from(args.dss), &args.pathname).map(|count| Deleted { pathname: args.pathname, count }))
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
struct IrregularArgs {
    dss: String,
    pathname: String,
    start_date: String,
    times: Vec<i32>,
    values: Vec<f64>,
    #[serde(default)]
    units: Option<String>,
    #[serde(default)]
    data_type: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct DeleteArgs {
    dss: String,
    pathname: String,
}

#[derive(Debug, serde::Serialize)]
struct Deleted {
    pathname: String,
    count: usize,
}
