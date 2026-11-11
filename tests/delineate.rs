use std::path::PathBuf;

use mcp_hechms::project::ProjectStore;
use mcp_hechms::terrain::delineate;

#[test]
fn delineates_the_cells_that_drain_to_the_outlet() {
    let dir = std::env::temp_dir().join("mcp-hechms-delineate");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let dem = dir.join("cuenca.asc");
    std::fs::write(
        &dem,
        "ncols 3\nnrows 3\nxllcorner 0\nyllcorner 0\ncellsize 1000\nNODATA_value -9999\n1 9 4\n9 9 3\n9 8 2\n",
    )
    .unwrap();
    let result = delineate(&dem, 2500.0, 500.0, "Sub-A", 0).unwrap();
    assert_eq!(result.cells, 6);
    assert!((result.area_km2 - 6.0).abs() < 1e-6, "{}", result.area_km2);
    assert!((result.outlet_x - 2500.0).abs() < 1e-6);
    assert!((result.outlet_y - 500.0).abs() < 1e-6);
    let geojson = std::fs::read_to_string(&result.geojson).unwrap();
    assert!(geojson.contains("Polygon"));

    let hms = dir.join("Demo.hms");
    std::fs::write(&hms, "Project: Demo\n     Version: 4.14\n     Unit System: Metric\nEnd:\n").unwrap();
    let store = ProjectStore::default();
    store.create_basin(&hms, "Cuenca", "Metric").unwrap();
    let area = format!("{:.4}", result.area_km2);
    store.create_element(&hms, Some("Cuenca"), "Subbasin", "Sub-A", "", &area).unwrap();
    let basin = store.basin(&hms, Some("Cuenca")).unwrap();
    assert!(basin.elements.iter().any(|item| item.name == "Sub-A" && item.area.as_deref() == Some(area.as_str())));
    let _ = PathBuf::from(dir);
}
