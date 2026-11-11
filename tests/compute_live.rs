use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use mcp_hechms::compute::ComputeHandle;
use mcp_hechms::config::Config;

#[tokio::test]
async fn compute_mirador_copy() {
    let source = Path::new(r"C:\Users\rsant\Documents\MIRADOR\MIRADOR.hms");
    if !source.is_file() {
        return;
    }
    let config = Config::resolve();
    if config.hms_home.is_none() {
        return;
    }
    let main = config.worker_classes.join("hmsmcp").join("Main.class");
    assert!(main.is_file(), "falta el worker compilado en {}", main.display());
    let copy = std::env::temp_dir().join("mcp-hechms-mirador");
    if copy.exists() {
        std::fs::remove_dir_all(&copy).expect("limpiar copia");
    }
    copy_tree(source.parent().unwrap(), &copy).expect("copiar proyecto");
    let hms = copy.join("MIRADOR.hms");
    let handle = Arc::new(ComputeHandle::new(Arc::new(config)));
    let first = handle.run(&hms, "Run 2", "run", Duration::from_secs(600)).await;
    let second = handle.run(&hms, "Run 2", "run", Duration::from_secs(600)).await;
    let first = first.expect_err("HEC-HMS debe informar la corrida abortada");
    let second = second.expect_err("la segunda llamada reutiliza el worker");
    assert!(first.to_string().contains("Subbasin-2"), "{first}");
    assert!(second.to_string().contains("Subbasin-2"), "{second}");
    let kinds = ["optimization", "forecast", "uncertainty", "frequency", "ensemble", "depth-area"];
    for kind in kinds {
        let result = handle.run(&hms, "No Existe", kind, Duration::from_secs(120)).await;
        let text = result.expect_err("el análisis inexistente debe fallar dentro de HEC-HMS").to_string();
        assert!(!text.contains("NoSuchMethod"), "{kind}: {text}");
        assert!(!text.contains("desconocido"), "{kind}: {text}");
    }
    handle.invalidate(&hms).await;
    let projects = mcp_hechms::project::ProjectStore::default();
    projects
        .apply_scs(
            &hms,
            Some("Basin 1"),
            mcp_hechms::project::ScsInput {
                element: "Subbasin-1",
                curve_number: 75.0,
                lag_minutes: 30.0,
                initial_abstraction: None,
                impervious_percent: None,
                unitgraph: None,
            },
        )
        .expect("número de curva");
    let with_scs = handle.run(&hms, "Run 2", "run", Duration::from_secs(180)).await;
    let text = with_scs.expect_err("la corrida sigue abortando por el pluviómetro").to_string();
    assert!(text.contains("Subbasin-2"), "{text}");
    assert!(!text.contains("Unknown"), "{text}");
    let basin = std::fs::read_to_string(copy.join("Basin_1.basin")).expect("leer cuenca");
    assert!(basin.contains("LossRate: SCS") && basin.contains("Curve Number:"), "{basin}");
    handle.invalidate(&hms).await;
    projects
        .create_analysis(&hms, "forecast", "Pronostico MCP", "Basin 1", "Met 1", "Control 1")
        .expect("crear pronóstico");
    let created = handle.run(&hms, "Pronostico MCP", "forecast", Duration::from_secs(180)).await;
    let text = created.expect_err("HEC-HMS debe abrir el pronóstico nuevo").to_string();
    assert!(text.contains("Subbasin-2"), "{text}");
}

fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let dest = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &dest)?;
        } else {
            std::fs::copy(entry.path(), dest)?;
        }
    }
    Ok(())
}
