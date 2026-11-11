use std::path::{Path, PathBuf};

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorData;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use super::result::pack;
use crate::error::HmsError;
use crate::server::HmsServer;

#[derive(Debug, Deserialize, JsonSchema)]
struct BasinArgs {
    project: String,
    name: String,
    #[serde(default)]
    units: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ControlArgs {
    project: String,
    name: String,
    start_date: String,
    #[serde(default)]
    start_time: Option<String>,
    end_date: String,
    #[serde(default)]
    end_time: Option<String>,
    #[serde(default)]
    interval: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct MetArgs {
    project: String,
    name: String,
    basin: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ElementArgs {
    project: String,
    #[serde(default)]
    basin: Option<String>,
    kind: String,
    name: String,
    #[serde(default)]
    downstream: Option<String>,
    #[serde(default)]
    area: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RunArgs {
    project: String,
    name: String,
    basin: String,
    precip: String,
    control: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct NamedArgs {
    project: String,
    name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AnalysisArgs {
    project: String,
    /// forecast, optimization, uncertainty, frequency, ensemble o depth-area
    kind: String,
    name: String,
    basin: String,
    precip: String,
    control: String,
}

#[tool_router(vis = "pub(crate)", router = create_model_router)]
impl HmsServer {
    #[tool(description = "Crea una cuenca vacía y la registra en el proyecto .hms.")]
    async fn hms_create_basin(&self, Parameters(args): Parameters<BasinArgs>) -> Result<String, ErrorData> {
        self.prepare(&args.project).await;
        self.saved(self.projects.create_basin(&PathBuf::from(&args.project), &args.name, args.units.as_deref().unwrap_or("Metric"))).await
    }

    #[tool(description = "Crea una especificación de control con fechas e intervalo y la registra en el proyecto.")]
    async fn hms_create_control(&self, Parameters(args): Parameters<ControlArgs>) -> Result<String, ErrorData> {
        self.prepare(&args.project).await;
        let path = PathBuf::from(&args.project);
        self.saved(self.projects.create_control(&path, &args.name, &args.start_date, args.start_time.as_deref().unwrap_or("00:00"), &args.end_date, args.end_time.as_deref().unwrap_or("24:00"), args.interval.as_deref().unwrap_or("15"))).await
    }

    #[tool(description = "Crea un modelo meteorológico de precipitación media y lo registra en el proyecto.")]
    async fn hms_create_met(&self, Parameters(args): Parameters<MetArgs>) -> Result<String, ErrorData> {
        self.prepare(&args.project).await;
        self.saved(self.projects.create_met(&PathBuf::from(&args.project), &args.name, &args.basin)).await
    }

    #[tool(description = "Agrega un elemento a la cuenca: Subbasin, Reach, Junction, Reservoir, Sink, Source o Diversion.")]
    async fn hms_create_element(&self, Parameters(args): Parameters<ElementArgs>) -> Result<String, ErrorData> {
        self.prepare(&args.project).await;
        self.saved(self.projects.create_element(&PathBuf::from(&args.project), args.basin.as_deref(), &args.kind, &args.name, args.downstream.as_deref().unwrap_or(""), args.area.as_deref().unwrap_or(""))).await
    }

    #[tool(description = "Crea una corrida que usa una cuenca, una meteorología y un control ya existentes.")]
    async fn hms_create_run(&self, Parameters(args): Parameters<RunArgs>) -> Result<String, ErrorData> {
        self.prepare(&args.project).await;
        self.saved(self.projects.create_run(&PathBuf::from(&args.project), &args.name, &args.basin, &args.precip, &args.control)).await
    }

    #[tool(description = "Crea un pluviómetro de precipitación incremental en el archivo de gages del proyecto.")]
    async fn hms_create_gage(&self, Parameters(args): Parameters<NamedArgs>) -> Result<String, ErrorData> {
        self.prepare(&args.project).await;
        self.saved(self.projects.create_gage(&PathBuf::from(&args.project), &args.name)).await
    }

    #[tool(description = "Crea un pronóstico, una optimización, una incertidumbre, una frecuencia, un ensamble o un análisis profundidad-área.")]
    async fn hms_create_analysis(&self, Parameters(args): Parameters<AnalysisArgs>) -> Result<String, ErrorData> {
        self.prepare(&args.project).await;
        self.saved(self.projects.create_analysis(&PathBuf::from(&args.project), &args.kind, &args.name, &args.basin, &args.precip, &args.control)).await
    }

    async fn prepare(&self, project: &str) {
        self.compute.invalidate(Path::new(project)).await;
    }

    async fn saved<T: serde::Serialize>(&self, result: Result<T, HmsError>) -> Result<String, ErrorData> {
        pack(result)
    }
}
