use std::path::PathBuf;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorData;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use super::result::pack;
use crate::server::HmsServer;

#[derive(Debug, Deserialize, JsonSchema)]
struct ReadArgs {
    /// Ruta del DSS 7
    dss: String,
    /// Pathname de la curva
    pathname: String,
    /// Máximo de ordenadas devueltas. Tope 500.
    #[serde(default)]
    limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct StoreArgs {
    /// Ruta del DSS 7. Si no existe, se crea.
    dss: String,
    /// Pathname de la curva
    pathname: String,
    /// Ordenadas independientes
    ordinates: Vec<f64>,
    /// Valores dependientes. Una curva: la misma cantidad que las ordenadas.
    values: Vec<f64>,
    /// Número de curvas. Por defecto 1.
    #[serde(default)]
    curves: Option<i32>,
    #[serde(default)]
    units_independent: Option<String>,
    #[serde(default)]
    type_independent: Option<String>,
    #[serde(default)]
    units_dependent: Option<String>,
    #[serde(default)]
    type_dependent: Option<String>,
}

#[tool_router(vis = "pub(crate)", router = dss_paired_router)]
impl HmsServer {
    #[tool(description = "Lee datos pareados de un DSS 7: ordenadas, curvas y unidades. Como máximo 500 ordenadas.")]
    async fn hms_dss_paired(&self, Parameters(args): Parameters<ReadArgs>) -> Result<String, ErrorData> {
        let limit = args.limit.unwrap_or(500).clamp(1, 500) as usize;
        pack(self.dss.paired(&PathBuf::from(args.dss), &args.pathname, limit))
    }

    #[tool(description = "Escribe una curva de datos pareados en un DSS 7 y devuelve lo guardado.")]
    async fn hms_dss_store_paired(&self, Parameters(args): Parameters<StoreArgs>) -> Result<String, ErrorData> {
        pack(self.dss.store_paired(
            &PathBuf::from(args.dss),
            &args.pathname,
            &args.ordinates,
            &args.values,
            args.curves.unwrap_or(1),
            args.units_independent.as_deref().unwrap_or("MM"),
            args.type_independent.as_deref().unwrap_or(""),
            args.units_dependent.as_deref().unwrap_or("M3/S"),
            args.type_dependent.as_deref().unwrap_or(""),
        ))
    }
}
