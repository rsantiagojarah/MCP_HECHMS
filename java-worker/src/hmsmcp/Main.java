package hmsmcp;

import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.io.PrintWriter;
import java.nio.charset.StandardCharsets;

public final class Main {
    private Main() {}

    public static void main(String[] args) throws Exception {
        Engine engine = new Engine();
        BufferedReader in = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8));
        PrintWriter out = new PrintWriter(new java.io.OutputStreamWriter(System.out, StandardCharsets.UTF_8), true);
        String line;
        while ((line = in.readLine()) != null) {
            if (line.isBlank()) {
                continue;
            }
            String cmd = Protocol.field(line, "cmd");
            try {
                switch (cmd) {
                    case "ping" -> out.println(Reply.ok(cmd, engine.ping()));
                    case "open" -> out.println(Reply.ok(cmd, engine.open(Protocol.field(line, "project"))));
                    case "compute" -> out.println(Reply.ok(cmd, engine.compute(Protocol.field(line, "kind"), Protocol.field(line, "run"))));
                    case "close" -> out.println(Reply.ok(cmd, engine.close()));
                    case "shutdown" -> {
                        out.println(Reply.ok(cmd, engine.close()));
                        return;
                    }
                    default -> out.println(Reply.fail(cmd, new IllegalArgumentException("unknown_command")));
                }
            } catch (Exception ex) {
                out.println(Reply.fail(cmd, ex));
            }
        }
    }
}
