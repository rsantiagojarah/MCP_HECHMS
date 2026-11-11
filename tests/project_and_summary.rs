use mcp_hechms::dss::{Sample, format_julian, summarize};
use mcp_hechms::project::parse;

const SAMPLE: &str = r#"
Project: Demo
     Description: Cuenca de prueba
     Version: 4.14
     DSS File Name: Demo.dss
     Time Zone ID: America/Lima
End:

Basin: Basin 1
     Filename: Basin_1.basin
     Description: Principal
End:

Control: Control 1
     Filename: Control_1.control
End:

Run: Current
     Basin: Basin 1
     Precipitation: Met 1
     Control: Control 1
End:
"#;

#[test]
fn parses_project_and_keeps_end_date_as_field() {
    let blocks = parse(SAMPLE);
    let project = blocks.iter().find(|block| block.key == "Project").unwrap();
    assert_eq!(project.value, "Demo");
    assert_eq!(project.field("Version"), Some("4.14"));
    let control = r#"
Control: Control 1
     Start Date: 01Jan2020
     Start Time: 0000
     End Date: 02Jan2020
     End Time: 2400
     Time Interval: 15
End:
"#;
    let blocks = parse(control);
    let control = &blocks[0];
    assert_eq!(control.field("End Date"), Some("02Jan2020"));
    assert_eq!(control.field("Time Interval"), Some("15"));
}

#[test]
fn parses_nested_basin_layers_and_elements() {
    let text = r#"
Basin: Basin 1
     Unit System: Metric
     Version: 4.14
End:

Basin Layer Properties:
     Base Layer:
          Name: Terrain
          Layer shown: Yes
     End Layer:
End:

Subbasin: Sub-1
     Area: 12.5
     Downstream: J-1
     LossRate: DeficitConstant
     Canvas X: 10
End:

Sink: Outlet
End:
"#;
    let blocks = parse(text);
    let layer = blocks.iter().find(|block| block.key == "Basin Layer Properties").unwrap();
    let base = layer.children.iter().find(|child| child.key == "Base Layer").unwrap();
    assert_eq!(base.field("Name"), Some("Terrain"));
    let info = mcp_hechms::project::basin_from(&blocks);
    assert_eq!(info.elements.len(), 2);
    assert_eq!(info.elements[0].name, "Sub-1");
    assert_eq!(info.elements[0].area.as_deref(), Some("12.5"));
    assert!(info.elements[0].parameters.iter().all(|field| field.name != "Canvas X"));
}

#[test]
fn julian_day_one_is_first_of_january_1900() {
    assert_eq!(format_julian(1, 0), "01Jan1900 00:00");
    assert_eq!(format_julian(25568, 3600), "01Jan1970 01:00");
}

#[test]
fn summary_reports_peak_and_integral() {
    let samples = [
        Sample { offset_seconds: 0, value: 1.0 },
        Sample { offset_seconds: 3600, value: 4.0 },
        Sample { offset_seconds: 7200, value: -1.0e38 },
    ];
    let summary = summarize("/A/B/FLOW/01Jan2020/1HOUR/RUN/", &samples, "m3/s", "PER-AVER", 3600, 1);
    assert_eq!(summary.peak, Some(4.0));
    assert_eq!(summary.valid_count, 2);
    assert_eq!(summary.integral_value_seconds, Some(5.0 * 3600.0));
    assert_eq!(summary.time_of_peak.as_deref(), Some("01Jan1900 01:00"));
}

#[test]
fn empty_a_part_keeps_the_date() {
    let (start, _, end, _) = mcp_hechms::dss::d_part_window("//Subbasin-3/FLOW/01Jan2000/15Minute/RUN:Run 2/");
    assert_eq!(start, "01Jan2000");
    assert_eq!(end, "01Jan2000");
    let stamp = mcp_hechms::dss::format_offset(0, 52_551_360 * 60);
    assert_eq!(stamp, "01Dec1999 00:00");
}

#[test]
fn control_filename_and_sidecar_run() {
    let dir = std::env::temp_dir().join("mcp-hechms-fixture");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("Demo.hms"),
        "Project: Demo\n     Version: 4.14\nEnd:\n\nControl: Control 1\n     FileName: Control_1.control\nEnd:\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("Control_1.control"),
        "Control: Control 1\n     Start Date: 01Jan2000\n     Start Time: 0000\n     End Date: 02Jan2000\n     End Time: 2400\n     Time Interval: 15\nEnd:\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("Demo.run"),
        "Run: Run 2\n     Basin: Basin 1\n     DSS File: Run_2.dss\nEnd:\n",
    )
    .unwrap();
    let info = mcp_hechms::project::ProjectStore::default()
        .info(&dir.join("Demo.hms"))
        .unwrap();
    assert_eq!(info.controls[0].start_date, "01Jan2000");
    assert!(info.components.iter().any(|item| item.kind == "Run" && item.name == "Run 2"));
}

#[test]
fn default_home_is_hms_414() {
    if std::env::var_os("HMS_HOME").is_some() {
        return;
    }
    let jar = std::path::Path::new(r"C:\Program Files\HEC\HEC-HMS\4.14\hms.jar");
    if !jar.is_file() {
        return;
    }
    let config = mcp_hechms::config::Config::resolve();
    assert_eq!(config.hms_home.as_deref(), jar.parent());
    assert!(config.java_exe.unwrap().ends_with("java.exe"));
}
