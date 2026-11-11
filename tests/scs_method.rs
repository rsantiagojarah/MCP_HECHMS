use mcp_hechms::project::{ProjectStore, ScsInput};

#[test]
fn applies_scs_curve_number_and_unit_hydrograph() {
    let dir = std::env::temp_dir().join("mcp-hechms-scs");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let hms = dir.join("Demo.hms");
    std::fs::write(&hms, "Project: Demo\nEnd:\n\nBasin: Cuenca\n     Filename: Cuenca.basin\nEnd:\n").unwrap();
    std::fs::write(
        dir.join("Cuenca.basin"),
        "Basin: Cuenca\n     Unit System: Metric\nEnd:\n\nSubbasin: Sub-A\n     Area: 1.2\n     LossRate: Initial+Constant\n     Constant Rate: 1\n     Transform: Clark\n     Clark Method: Specified\n     Baseflow: None\nEnd:\n",
    )
    .unwrap();
    let store = ProjectStore::default();
    let applied = store
        .apply_scs(&hms, Some("Cuenca"), input("Sub-A", 75.0, 30.0))
        .unwrap();
    assert_eq!(applied.curve_number, "75");
    assert_eq!(applied.initial_abstraction, "16.9333");
    assert_eq!(applied.lag_minutes, "30");
    assert_eq!(applied.unitgraph, "STANDARD");
    let text = std::fs::read_to_string(dir.join("Cuenca.basin")).unwrap();
    assert!(text.contains("LossRate: SCS"));
    assert!(text.contains("Curve Number: 75"));
    assert!(text.contains("Initial Abstraction: 16.9333"));
    assert!(text.contains("Transform: SCS"));
    assert!(text.contains("Unitgraph Type: STANDARD"));
    assert!(text.contains("Lag: 30"));
    assert!(text.contains("Baseflow: None"));
    assert!(!text.contains("Clark Method"));
    assert!(!text.contains("Constant Rate"));
    let basin = store.basin(&hms, Some("Cuenca")).unwrap();
    let element = basin.elements.iter().find(|item| item.name == "Sub-A").unwrap();
    assert!(element.parameters.iter().any(|item| item.name == "Curve Number" && item.value == "75"));

    std::fs::write(
        dir.join("Cuenca.basin"),
        "Basin: Cuenca\n     Unit System: English\nEnd:\n\nSubbasin: Sub-A\n     LossRate: None\n     Transform: None\nEnd:\n",
    )
    .unwrap();
    let english = store.apply_scs(&hms, Some("Cuenca"), input("Sub-A", 80.0, 12.0)).unwrap();
    assert_eq!(english.initial_abstraction, "0.5");
}

fn input(element: &str, curve_number: f64, lag_minutes: f64) -> ScsInput<'_> {
    ScsInput {
        element,
        curve_number,
        lag_minutes,
        initial_abstraction: None,
        impervious_percent: None,
        unitgraph: None,
    }
}
