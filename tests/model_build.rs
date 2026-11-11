use std::path::PathBuf;

use mcp_hechms::project::ProjectStore;

#[test]
fn builds_a_model_and_edits_control_and_met() {
    let dir = std::env::temp_dir().join("mcp-hechms-build");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let hms = dir.join("Demo.hms");
    std::fs::write(&hms, "Project: Demo\n     Version: 4.14\n     Time Zone ID: America/Lima\nEnd:\n").unwrap();
    let store = ProjectStore::default();
    store.create_basin(&hms, "Cuenca Nueva", "Metric").unwrap();
    store.create_control(&hms, "Control Nuevo", "1 January 2000", "00:00", "2 January 2000", "24:00", "15").unwrap();
    store.create_met(&hms, "Met Nueva", "Cuenca Nueva").unwrap();
    store.create_element(&hms, Some("Cuenca Nueva"), "Subbasin", "Sub-A", "J-1", "1.5").unwrap();
    store.create_element(&hms, Some("Cuenca Nueva"), "Junction", "J-1", "", "").unwrap();
    store.create_run(&hms, "Corrida Nueva", "Cuenca Nueva", "Met Nueva", "Control Nuevo").unwrap();
    store.create_gage(&hms, "Pluvio 1").unwrap();
    store.create_analysis(&hms, "forecast", "Pronostico 1", "Cuenca Nueva", "Met Nueva", "Control Nuevo").unwrap();
    store.create_analysis(&hms, "optimization", "Trial 1", "Cuenca Nueva", "Met Nueva", "Control Nuevo").unwrap();
    store.create_analysis(&hms, "uncertainty", "UA 1", "Cuenca Nueva", "Met Nueva", "Control Nuevo").unwrap();
    store.create_analysis(&hms, "frequency", "Freq 1", "Cuenca Nueva", "Met Nueva", "Control Nuevo").unwrap();
    store.create_analysis(&hms, "ensemble", "Ens 1", "Cuenca Nueva", "Met Nueva", "Control Nuevo").unwrap();
    store.create_analysis(&hms, "depth-area", "DA 1", "Cuenca Nueva", "Met Nueva", "Control Nuevo").unwrap();
    let change = store.set_in(&hms, "control", Some("Control Nuevo"), "Control Nuevo", "Start Date", "3 January 2000").unwrap();
    assert_eq!(change.previous, "1 January 2000");
    let met = store.set_in(&hms, "met", Some("Met Nueva"), "Met Nueva", "Precipitation Method", "Specified Hyetograph").unwrap();
    assert_eq!(met.value, "Specified Hyetograph");
    let info = store.info(&hms).unwrap();
    for name in ["Pronostico 1", "Trial 1", "UA 1", "Freq 1", "Ens 1", "DA 1", "Corrida Nueva", "Cuenca Nueva"] {
        assert!(info.components.iter().any(|item| item.name == name), "{name} no aparece");
    }
    let basin = store.basin(&hms, Some("Cuenca Nueva")).unwrap();
    assert!(basin.elements.iter().any(|item| item.name == "Sub-A" && item.area.as_deref() == Some("1.5")));
    assert!(dir.join("Pluvio_1.gage").exists() || dir.join("Demo.gage").exists());
    let _ = PathBuf::from(&dir);
}
