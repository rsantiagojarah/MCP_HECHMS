mod basin;
mod compute;
mod create_model;
mod delineate;
mod dss_catalog;
mod dss_grid;
mod dss_paired;
mod dss_store;
mod ping;
mod project_info;
mod result;
mod series_summary;
mod scs;
mod series_window;
mod set_parameter;

use rmcp::handler::server::router::tool::ToolRouter;

use crate::server::HmsServer;

/// Cada herramienta nueva se declara aquí y en su archivo. El resto del servidor no cambia.
pub fn routers() -> ToolRouter<HmsServer> {
    HmsServer::ping_router()
        + HmsServer::project_info_router()
        + HmsServer::basin_router()
        + HmsServer::dss_catalog_router()
        + HmsServer::series_summary_router()
        + HmsServer::series_window_router()
        + HmsServer::compute_router()
        + HmsServer::set_parameter_router()
        + HmsServer::dss_store_router()
        + HmsServer::dss_paired_router()
        + HmsServer::dss_grid_router()
        + HmsServer::create_model_router()
        + HmsServer::scs_router()
        + HmsServer::delineate_router()
}
