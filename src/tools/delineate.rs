use std::path::{Path, PathBuf};

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorData;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use super::result::pack;
use crate::project::ProjectStore;
use crate::server::HmsServer;

#[derive(Debug, Deserialize, JsonSchema)]
struct Args {
    /// Ruta del modelo de elevación: ASCII grid (.asc) o GeoTIFF (.tif)
    dem: String,
    /// Coordenada X del punto de salida, en el sistema del modelo
    outlet_x: f64,
    /// Coordenada Y del punto de salida, en el sistema del modelo
    outlet_y: f64,
    /// Proyecto .hms donde se agrega la subcuenca. Si se omite, solo se delimita y se escribe el GeoJSON
    #[serde(default)]
    project: Option<String>,
    /// Cuenca de destino. Si el proyecto no tiene una, se crea Cuenca
    #[serde(default)]
    basin: Option<String>,
    /// Nombre de la subcuenca delimitada
    #[serde(default)]
    element: Option<String>,
    /// Celdas alrededor del punto para moverlo al cauce. 0 lo deja donde está
    #[serde(default)]
    snap_cells: Option<u32>,
}

#[tool_router(vis = "pub(crate)", router = delineate_router)]
impl HmsServer {
    #[tool(description = "Delimita la cuenca que drena a un punto de salida sobre un modelo de elevación. Guarda el contorno GeoJSON y, si hay un proyecto, crea la subcuenca con su área.")]
    async fn hms_delineate_basin(&self, Parameters(args): Parameters<Args>) -> Result<String, ErrorData> {
        let dem = PathBuf::from(&args.dem);
        let element = args.element.unwrap_or_else(|| "Subbasin-1".into());
        let mut result = match crate::terrain::delineate(&dem, args.outlet_x, args.outlet_y, &element, args.snap_cells.unwrap_or(0)) {
            Ok(value) => value,
            Err(err) => return pack::<crate::terrain::Delineation>(Err(err)),
        };
        if let Some(project) = args.project.as_deref() {
            self.compute.invalidate(Path::new(project)).await;
            match attach(&self.projects, Path::new(project), args.basin.as_deref(), &element, result.area_km2) {
                Ok(name) => result.element = name,
                Err(err) => return pack::<crate::terrain::Delineation>(Err(err)),
            }
        }
        pack(Ok(result))
    }
}

fn attach(store: &ProjectStore, project: &Path, basin: Option<&str>, element: &str, area_km2: f64) -> Result<String, crate::error::HmsError> {
    let basin_name = basin.map(str::to_string).or_else(|| existing_basin(store, project)).unwrap_or_else(|| "Cuenca".into());
    if store.create_basin(project, &basin_name, "Metric").is_err() && existing_basin(store, project).is_none() {
        return Err(crate::error::HmsError::message("no se pudo crear la cuenca en el proyecto"));
    }
    let english = store.basin(project, Some(&basin_name)).map(|item| item.unit_system.eq_ignore_ascii_case("English")).unwrap_or(false);
    let area = if english { area_km2 * 0.386102 } else { area_km2 };
    let area = format!("{area:.4}");
    match store.create_element(project, Some(&basin_name), "Subbasin", element, "", &area) {
        Ok(created) => Ok(created.name),
        Err(_) => {
            store.set_in(project, "basin", Some(&basin_name), element, "Area", &area)?;
            Ok(element.to_string())
        }
    }
}

fn existing_basin(store: &ProjectStore, project: &Path) -> Option<String> {
    store.info(project).ok()?.components.into_iter().find(|item| item.kind == "Basin").map(|item| item.name)
}
