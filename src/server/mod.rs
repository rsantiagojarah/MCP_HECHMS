use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::{tool_handler, ServerHandler, ServiceExt};

use crate::compute::ComputeHandle;
use crate::config::Config;
use crate::dss::DssPool;
use crate::project::ProjectStore;
use crate::tools;

#[derive(Clone)]
pub struct HmsServer {
    tool_router: ToolRouter<Self>,
    pub(crate) config: Arc<Config>,
    pub(crate) projects: Arc<ProjectStore>,
    pub(crate) dss: Arc<DssPool>,
    pub(crate) compute: Arc<ComputeHandle>,
}

#[tool_handler(
    router = self.tool_router,
    name = "hec-hms",
    instructions = "Servidor de HEC-HMS 4.14. Crea y edita cuencas, controles, meteorología, corridas y análisis. Delimita una cuenca desde un modelo de elevación y un punto de salida. Configura el número de curva del SCS y el hidrograma unitario del SCS. Lee y escribe series, curvas y grillas DSS 7. Calcula corridas, optimización, pronóstico, incertidumbre, frecuencia, ensamble y profundidad-área."
)]
impl ServerHandler for HmsServer {}

impl HmsServer {
    pub fn new(config: Arc<Config>) -> Self {
        let compute = Arc::new(ComputeHandle::new(Arc::clone(&config)));
        Self {
            tool_router: tools::routers(),
            projects: Arc::new(ProjectStore::default()),
            dss: Arc::new(DssPool::new(&config)),
            compute,
            config,
        }
    }

    pub fn warm(&self) {
        self.compute.warm();
    }
}

pub async fn run() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let config = Arc::new(Config::resolve());
    let server = HmsServer::new(config);
    server.warm();
    let running = server.serve(rmcp::transport::stdio()).await?;
    running.waiting().await?;
    Ok(())
}
