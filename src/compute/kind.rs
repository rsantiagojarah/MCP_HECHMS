use crate::error::HmsError;

pub fn canonical(kind: &str) -> Result<&'static str, HmsError> {
    let text = kind.trim();
    let folded = text.to_ascii_lowercase();
    match folded.as_str() {
        "" | "run" | "simulation" | "corrida" => Ok("run"),
        "optimization" | "optimizacion" | "optimización" | "trial" => Ok("optimization"),
        "forecast" | "pronostico" | "pronóstico" => Ok("forecast"),
        "uncertainty" | "incertidumbre" => Ok("uncertainty"),
        "frequency" | "frecuencia" => Ok("frequency"),
        "ensemble" | "ensamble" => Ok("ensemble"),
        "depth-area" | "deptharea" | "profundidad-area" | "profundidad-área" => Ok("depth-area"),
        _ => Err(HmsError::message(
            "tipo de cálculo desconocido. Use run, optimization, forecast, uncertainty, frequency, ensemble o depth-area",
        )),
    }
}
