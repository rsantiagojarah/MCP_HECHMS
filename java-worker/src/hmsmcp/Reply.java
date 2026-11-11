package hmsmcp;

final class Reply {
    private Reply() {}

    static String ok(String cmd, String fields) {
        StringBuilder text = new StringBuilder();
        text.append("{\"ok\":true,\"cmd\":\"").append(escape(cmd)).append('"');
        if (fields != null && !fields.isBlank()) {
            text.append(',').append(fields);
        }
        text.append('}');
        return text.toString();
    }

    static String fail(String cmd, Throwable error) {
        String message = error.getMessage() == null ? error.toString() : error.getMessage();
        return "{\"ok\":false,\"cmd\":\"" + escape(cmd) + "\",\"error\":\"" + escape(message) + "\"}";
    }

    static String escape(String value) {
        if (value == null) {
            return "";
        }
        StringBuilder out = new StringBuilder(value.length() + 8);
        for (int i = 0; i < value.length(); i++) {
            char ch = value.charAt(i);
            switch (ch) {
                case '\\' -> out.append("\\\\");
                case '"' -> out.append("\\\"");
                case '\n' -> out.append("\\n");
                case '\r' -> out.append("\\r");
                case '\t' -> out.append("\\t");
                default -> out.append(ch);
            }
        }
        return out.toString();
    }
}
