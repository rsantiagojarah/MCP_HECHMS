use std::path::Path;

use mcp_hechms::config::Config;
use mcp_hechms::dss::{DssPool, TimeWindow};

#[test]
fn catalog_and_summary_of_run_2() {
    let dss = Path::new(r"C:\Users\rsant\Documents\MIRADOR\Run_2.dss");
    if !dss.is_file() {
        return;
    }
    let config = Config::resolve();
    let Some(dll) = &config.hecdss_dll else {
        return;
    };
    assert!(dll.ends_with("hecdss.dll"));
    let pool = DssPool::new(&config);
    let page = pool.catalog(dss, "/*/*/FLOW/*/*/*/", 5).expect("catálogo");
    assert!(page.total >= 5, "total {}", page.total);
    assert!(page.pathnames.iter().any(|path| path.contains("FLOW")));
    let summary = pool
        .summary(
            dss,
            "//Subbasin-3/FLOW/01Jan2000/15Minute/RUN:Run 2/",
            &TimeWindow::resolve("//Subbasin-3/FLOW/01Jan2000/15Minute/RUN:Run 2/", "", "", "", ""),
        )
        .expect("resumen");
    assert_eq!(summary.units, "M3/S");
    assert!(summary.count > 0);
    assert_eq!(summary.interval_seconds, 900);
}
