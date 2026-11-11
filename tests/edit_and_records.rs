use std::path::PathBuf;

use mcp_hechms::config::Config;
use mcp_hechms::dss::{DssPool, TimeWindow};
use mcp_hechms::project::ProjectStore;

#[test]
fn edits_a_parameter_and_lists_analyses() {
    let dir = std::env::temp_dir().join("mcp-hechms-edit");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("Demo.hms"),
        "Project: Demo\n     Version: 4.14\nEnd:\n\nBasin: Basin 1\n     Filename: Basin_1.basin\nEnd:\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("Basin_1.basin"),
        "Basin: Basin 1\n     Unit System: Metric\nEnd:\n\nSubbasin: Subbasin-1\n     Area: 0.25\n     Lag: 2\n     Last Modified Date: 7 November 2024\n     Last Modified Time: 17:41:30\nEnd:\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("Pronostico.forecast"),
        "Forecast Alternative: Pronostico 1\n     Basin: Basin 1\nEnd:\n",
    )
    .unwrap();
    std::fs::write(dir.join("Trial.optim"), "Optimization Trial: Trial 1\n     Basin: Basin 1\nEnd:\n").unwrap();
    std::fs::create_dir_all(dir.join("montecarlo")).unwrap();
    std::fs::write(
        dir.join("montecarlo").join("Ua.mca"),
        "Monte Carlo Analysis: UA 1\n     Basin Model Name: Basin 1\nEnd Monte Carlo Analysis:\n",
    )
    .unwrap();
    let store = ProjectStore::default();
    let hms = dir.join("Demo.hms");
    let change = store.set_parameter(&hms, None, "Subbasin-1", "Lag", "9").unwrap();
    assert_eq!(change.previous, "2");
    assert_eq!(change.value, "9");
    let basin = store.basin(&hms, None).unwrap();
    let lag = basin.elements[0].parameters.iter().find(|item| item.name == "Lag").unwrap();
    assert_eq!(lag.value, "9");
    let info = store.info(&hms).unwrap();
    assert!(info.components.iter().any(|item| item.kind == "Forecast Alternative" && item.name == "Pronostico 1"));
    assert!(info.components.iter().any(|item| item.kind == "Optimization Trial" && item.name == "Trial 1"));
    assert!(info.components.iter().any(|item| item.kind == "Monte Carlo Analysis" && item.name == "UA 1"));
}

#[test]
fn writes_series_paired_data_and_grid() {
    let config = Config::resolve();
    if config.hecdss_dll.is_none() {
        return;
    }
    let path = PathBuf::from(std::env::temp_dir().join("mcp-hechms-write.dss"));
    let _ = std::fs::remove_file(&path);
    let pool = DssPool::new(&config);
    let pathname = "/MCP/SITE/FLOW/01Jan2020/1Hour/TEST/";
    let stored = pool
        .store_series(&path, pathname, "01Jan2020", "0000", &[1.0, 2.5, 0.5], "M3/S", "INST-VAL")
        .expect("escribir serie");
    assert_eq!(stored.count, 3);
    let summary = pool
        .summary(&path, pathname, &TimeWindow::resolve(pathname, "01Jan2020", "0000", "01Jan2020", "2400"))
        .expect("leer serie");
    assert_eq!(summary.units, "M3/S");
    assert_eq!(summary.peak, Some(2.5));
    let curve = "/MCP/CURVE/FLOW-UNIT///TEST/";
    let paired = pool
        .store_paired(&path, curve, &[0.0, 1.0, 2.0], &[0.0, 10.0, 20.0], 1, "MM", "", "M3/S", "")
        .expect("escribir curva");
    assert_eq!(paired.ordinates, vec![0.0, 1.0, 2.0]);
    assert_eq!(paired.values, vec![0.0, 10.0, 20.0]);
    let grid = pool
        .store_grid(&path, "/MCP/GRID/PRECIP///TEST/", 2, 2, 2000.0, 410, "HRAP", "MM", &[1.0, 2.0, 1.5, 0.5])
        .expect("escribir grilla");
    assert_eq!(grid.grid_type, 410);
    let irregular = pool
        .store_irregular(&path, "/MCP/SITE/FLOW//IR-DAY/TEST/", "01Jan2020", &[0, 60, 180], &[1.0, 4.0, 2.0], "M3/S", "INST-VAL")
        .expect("serie irregular");
    assert_eq!(irregular.count, 3);
    let removed = pool.delete(&path, "/MCP/SITE/FLOW//IR-DAY/TEST/").expect("borrar registro");
    assert!(removed >= 1);
    let left = pool.catalog(&path, "/*/*/*/*/*/*/", 50).expect("catálogo final");
    assert!(left.pathnames.iter().all(|item| !item.to_ascii_lowercase().contains("ir-day")));
    assert_eq!(grid.cells_x, 2);
    assert_eq!(grid.cells_y, 2);
    assert!(grid.sample.iter().any(|value| (*value - 2.0).abs() < 0.01));
}
