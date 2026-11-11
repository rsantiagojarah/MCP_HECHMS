use crate::error::HmsError;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ParameterChange {
    pub element: String,
    pub parameter: String,
    pub previous: String,
    pub value: String,
}

pub fn replace_parameter(text: &str, element: &str, parameter: &str, value: &str) -> Result<(String, ParameterChange), HmsError> {
    if element.trim().is_empty() || parameter.trim().is_empty() {
        return Err(HmsError::message("faltan el elemento o el parámetro"));
    }
    if value.contains(['\n', '\r']) || parameter.contains(':') {
        return Err(HmsError::message("el parámetro o el valor no pueden ocupar más de una línea"));
    }
    let mut lines: Vec<String> = split_keep(text).into_iter().map(str::to_string).collect();
    let (start, end) = block_range(&lines, element)?;
    let previous = match find_field(&lines, start, end, parameter) {
        Some(index) => {
            let (prefix, previous) = split_field(&lines[index]);
            let ending = line_ending(&lines[index]);
            lines[index] = format!("{prefix}{value}{ending}");
            previous
        }
        None => {
            let ending = line_ending(&lines[end]);
            lines.insert(end, format!("     {parameter}: {value}{ending}"));
            String::new()
        }
    };
    let (date, clock) = super::stamp::date_time();
    let end = block_range(&lines, element)?.1;
    replace_field(&mut lines, start, end, "Last Modified Date", &date);
    replace_field(&mut lines, start, end, "Last Modified Time", &clock);
    Ok((
        lines.concat(),
        ParameterChange {
            element: element.to_string(),
            parameter: parameter.to_string(),
            previous,
            value: value.to_string(),
        },
    ))
}

fn replace_field(lines: &mut [String], start: usize, end: usize, parameter: &str, value: &str) {
    let Some(index) = find_field(lines, start, end, parameter) else {
        return;
    };
    let (prefix, _) = split_field(&lines[index]);
    let ending = line_ending(&lines[index]);
    lines[index] = format!("{prefix}{value}{ending}");
}

fn block_range(lines: &[String], element: &str) -> Result<(usize, usize), HmsError> {
    let mut found = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let Some((key, name)) = top_pair(line) else {
            continue;
        };
        if name.eq_ignore_ascii_case(element) || format!("{key}: {name}").eq_ignore_ascii_case(element) {
            found.push(index);
        }
    }
    let start = match found.as_slice() {
        [only] => *only,
        [] => return Err(HmsError::message(format!("no está el elemento {element}"))),
        _ => return Err(HmsError::message(format!("el elemento {element} está repetido"))),
    };
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find_map(|(index, line)| top_pair(line).filter(|(key, value)| *key == "End" && value.is_empty()).map(|_| index))
        .ok_or_else(|| HmsError::message(format!("el elemento {element} no tiene End:")))?;
    Ok((start, end))
}

fn find_field(lines: &[String], start: usize, end: usize, parameter: &str) -> Option<usize> {
    lines.iter().enumerate().take(end).skip(start + 1).find_map(|(index, line)| {
        let raw = strip_ending(line);
        let trimmed = raw.trim_start();
        if trimmed.is_empty() || !raw.starts_with([' ', '\t']) {
            return None;
        }
        let (key, _) = trimmed.split_once(':')?;
        key.trim().eq_ignore_ascii_case(parameter).then_some(index)
    })
}

fn split_field(line: &str) -> (String, String) {
    let raw = strip_ending(line);
    let Some((left, right)) = raw.split_once(':') else {
        return (raw.to_string(), String::new());
    };
    (format!("{left}: "), right.trim().to_string())
}

fn top_pair(line: &str) -> Option<(&str, &str)> {
    let raw = strip_ending(line);
    if raw.is_empty() || raw.starts_with([' ', '\t']) {
        return None;
    }
    let (key, value) = raw.split_once(':')?;
    Some((key.trim(), value.trim()))
}

fn strip_ending(line: &str) -> &str {
    line.trim_end_matches(['\n', '\r'])
}

fn line_ending(line: &str) -> &str {
    if line.ends_with("\r\n") { "\r\n" } else if line.ends_with('\n') { "\n" } else { "" }
}

fn split_keep(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    for (index, byte) in text.char_indices() {
        if byte == '\n' {
            lines.push(&text[start..=index]);
            start = index + 1;
        }
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}
