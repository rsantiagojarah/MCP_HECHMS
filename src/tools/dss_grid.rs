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
    /// Pathname de la grilla
    pathname: String,
    /// Primera celda de la página. Por defecto 0.
    #[serde(default)]
    offset: Option<u32>,
    /// Celdas de la página. Tope 500.
    #[serde(default)]
    limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct StoreArgs {
    /// Ruta del DSS 7. Si no existe, se crea.
    dss: String,
    /// Pathname de la grilla
    pathname: String,
    /// Celdas en X
    cells_x: i32,
    /// Celdas en Y
    cells_y: i32,
    /// Tamaño de celda
    cell_size: f32,
    /// shg, hrap, albers o undefined. Por defecto shg.
    #[serde(default)]
    grid_type: Option<String>,
    /// Unidades de los valores. Por defecto MM.
    #[serde(default)]
    units: Option<String>,
    /// Valores por fila, de izquierda a derecha. Como máximo 20000 celdas.
    values: Vec<f64>,
}

#[tool_router(vis = "pub(crate)", router = dss_grid_router)]
impl HmsServer {
    #[tool(description = "Resumen de una grilla DSS 7 y una página de hasta 500 celdas. No devuelve la grilla completa.")]
    async fn hms_dss_grid(&self, Parameters(args): Parameters<ReadArgs>) -> Result<String, ErrorData> {
        let offset = args.offset.unwrap_or(0) as usize;
        let limit = args.limit.unwrap_or(500).clamp(1, 500) as usize;
        pack(self.dss.grid(&PathBuf::from(args.dss), &args.pathname, offset, limit))
    }

    #[tool(description = "Escribe una grilla DSS 7 de hasta 20000 celdas. grid_type: shg, hrap, albers o undefined.")]
    async fn hms_dss_store_grid(&self, Parameters(args): Parameters<StoreArgs>) -> Result<String, ErrorData> {
        let (kind, srs) = match grid_kind(args.grid_type.as_deref().unwrap_or("shg")) {
            Ok(pair) => pair,
            Err(err) => return pack::<()>(Err(err)),
        };
        let values: Vec<f32> = args.values.iter().map(|value| *value as f32).collect();
        pack(self.dss.store_grid(
            &PathBuf::from(args.dss),
            &args.pathname,
            args.cells_x,
            args.cells_y,
            args.cell_size,
            kind,
            srs,
            args.units.as_deref().unwrap_or("MM"),
            &values,
        ))
    }
}

fn grid_kind(kind: &str) -> Result<(i32, &'static str), crate::error::HmsError> {
    match kind.trim().to_ascii_lowercase().as_str() {
        "" | "shg" => Ok((430, "SHG")),
        "hrap" => Ok((410, "HRAP")),
        "albers" => Ok((420, "ALBERS")),
        "undefined" | "indefinida" => Ok((400, "UNDEFINED")),
        _ => Err(crate::error::HmsError::message("tipo de grilla desconocido. Use shg, hrap, albers o undefined")),
    }
}
