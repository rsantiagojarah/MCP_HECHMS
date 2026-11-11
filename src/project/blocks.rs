/// Árbol genérico del formato de texto de HEC-HMS (`Clave: valor` y `End:`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub key: String,
    pub value: String,
    pub children: Vec<Block>,
}

struct Frame {
    indent: usize,
    block: Block,
}

pub fn parse(text: &str) -> Vec<Block> {
    let mut roots = Vec::new();
    let mut stack: Vec<Frame> = Vec::new();
    for raw in text.lines() {
        let indent = leading_spaces(raw);
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let Some((key, value)) = split_key(line) else {
            continue;
        };
        if is_closer(&key, &value) {
            close_until(&mut stack, &mut roots, indent);
            continue;
        }
        while stack.last().is_some_and(|frame| frame.indent >= indent) {
            pop(&mut stack, &mut roots);
        }
        stack.push(Frame {
            indent,
            block: Block {
                key,
                value,
                children: Vec::new(),
            },
        });
    }
    while !stack.is_empty() {
        pop(&mut stack, &mut roots);
    }
    roots
}

impl Block {
    pub fn fields(&self) -> impl Iterator<Item = (&str, &str)> {
        self.children
            .iter()
            .filter(|child| child.children.is_empty())
            .map(|child| (child.key.as_str(), child.value.as_str()))
    }

    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value)
    }

    pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Block> + 'a {
        self.children.iter().filter(move |child| child.key == name)
    }
}

fn pop(stack: &mut Vec<Frame>, roots: &mut Vec<Block>) {
    let Some(frame) = stack.pop() else {
        return;
    };
    if let Some(parent) = stack.last_mut() {
        parent.block.children.push(frame.block);
    } else {
        roots.push(frame.block);
    }
}

fn close_until(stack: &mut Vec<Frame>, roots: &mut Vec<Block>, indent: usize) {
    while stack.last().is_some_and(|frame| frame.indent >= indent) {
        pop(stack, roots);
    }
}

fn split_key(line: &str) -> Option<(String, String)> {
    let (key, value) = line.split_once(':')?;
    Some((key.trim().to_string(), value.trim().to_string()))
}

fn is_closer(key: &str, value: &str) -> bool {
    value.is_empty() && (key == "End" || key.starts_with("End "))
}

fn leading_spaces(raw: &str) -> usize {
    raw.chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum()
}
