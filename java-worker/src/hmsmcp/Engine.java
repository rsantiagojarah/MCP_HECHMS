package hmsmcp;

import java.lang.reflect.Method;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;

final class Engine {
    private Object project;
    private String projectPath = "";

    String ping() {
        return "\"message\":\"java " + Reply.escape(System.getProperty("java.version")) + "\"";
    }

    String open(String path) throws Exception {
        if (path == null || path.isBlank()) {
            throw new IllegalArgumentException("falta la ruta del proyecto");
        }
        if (path.equals(projectPath) && project != null) {
            return "\"message\":\"ya abierto\"";
        }
        close();
        Class<?> type = Class.forName("hms.model.Project");
        Method method = type.getMethod("open", String.class);
        project = method.invoke(null, path);
        projectPath = path;
        return "\"message\":\"abierto\"";
    }

    String compute(String kind, String run) throws Exception {
        if (project == null) {
            throw new IllegalStateException("no hay un proyecto abierto");
        }
        if (run == null || run.isBlank()) {
            throw new IllegalArgumentException("falta el nombre del cálculo");
        }
        if (kind == null || kind.isBlank()) {
            kind = "run";
        }
        String methodName = switch (kind) {
            case "optimization" -> "computeOptimization";
            case "forecast" -> "computeForecast";
            case "uncertainty" -> "computeUncertaintyAnalysis";
            case "frequency" -> "computeFrequencyAnalysis";
            case "ensemble" -> "computeEnsembleAnalysis";
            case "depth-area" -> "computeDepthAreaAnalysis";
            case "run" -> "computeRun";
            default -> throw new IllegalArgumentException("tipo de cálculo desconocido: " + kind);
        };
        long started = System.nanoTime();
        Method method = project.getClass().getMethod(methodName, String.class);
        try {
            method.invoke(project, run);
        } catch (java.lang.reflect.InvocationTargetException ex) {
            Throwable cause = ex.getCause() == null ? ex : ex.getCause();
            String message = cause.getMessage();
            if (message == null || message.isBlank()) {
                message = cause.toString();
            }
            throw new IllegalStateException(message, cause);
        }
        long elapsed = (System.nanoTime() - started) / 1_000_000L;
        String tail = logTail();
        if (tail.contains("Aborted ")) {
            throw new IllegalStateException(tail);
        }
        return "\"elapsed_ms\":" + elapsed + ",\"log_tail\":\"" + Reply.escape(tail) + "\"";
    }

    String close() throws Exception {
        if (project != null) {
            Method method = project.getClass().getMethod("close");
            method.invoke(project);
        }
        project = null;
        projectPath = "";
        return "\"message\":\"cerrado\"";
    }

    private String logTail() {
        Path log = newestLog();
        if (log == null) {
            return "";
        }
        try {
            List<String> lines = Files.readAllLines(log);
            int from = Math.max(0, lines.size() - 30);
            return String.join("\n", lines.subList(from, lines.size()));
        } catch (Exception ex) {
            return ex.getMessage() == null ? "" : ex.getMessage();
        }
    }

    private Path newestLog() {
        if (projectPath.isBlank()) {
            return null;
        }
        Path dir = Path.of(projectPath).getParent();
        if (dir == null) {
            return null;
        }
        try (var stream = Files.list(dir)) {
            Path best = null;
            long stamp = Long.MIN_VALUE;
            for (Path path : stream.toList()) {
                if (!path.getFileName().toString().endsWith(".log") || !Files.isRegularFile(path)) {
                    continue;
                }
                long modified = Files.getLastModifiedTime(path).toMillis();
                if (modified >= stamp) {
                    stamp = modified;
                    best = path;
                }
            }
            return best;
        } catch (Exception ex) {
            return null;
        }
    }
}
