package hmsmcp;

final class Protocol {
    private Protocol() {}

    static String field(String json, String key) {
        String needle = "\"" + key + "\"";
        int name = json.indexOf(needle);
        if (name < 0) {
            return "";
        }
        int colon = json.indexOf(':', name + needle.length());
        if (colon < 0) {
            return "";
        }
        int cursor = colon + 1;
        while (cursor < json.length() && Character.isWhitespace(json.charAt(cursor))) {
            cursor++;
        }
        if (cursor >= json.length() || json.charAt(cursor) != '"') {
            return "";
        }
        StringBuilder value = new StringBuilder();
        for (int i = cursor + 1; i < json.length(); i++) {
            char ch = json.charAt(i);
            if (ch == '\\' && i + 1 < json.length()) {
                value.append(json.charAt(i + 1));
                i++;
                continue;
            }
            if (ch == '"') {
                break;
            }
            value.append(ch);
        }
        return value.toString();
    }
}
