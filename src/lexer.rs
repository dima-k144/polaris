use crate::types::Error;

pub fn scan(source: &str) -> Result<Vec<(String, usize)>, Error> {
    let mut tokens = Vec::<(String, usize)>::new();
    let mut item = String::new();
    let mut line = 0;
    let mut in_string = false;
    let mut in_comment = false;
    let mut escape = false;

    for c in source.chars() {
        if in_comment {
            if c == '\n' {
                line += 1;
                in_comment = false;
            }
            continue;
        }

        if in_string {
            if escape {
                match c {
                    'n' => item.push('\n'),
                    't' => item.push('\t'),
                    'r' => item.push('\r'),
                    '\\' => item.push('\\'),
                    '"' => item.push('"'),
                    '0' => item.push('\0'),
                    // Optionally handle invalid escapes or pass through:
                    _ => item.push(c),
                }
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_string = false;
                item.push('"');
                tokens.push((item, line));
                item = String::new();
            } else {
                item.push(c);
            }
            continue;
        }

        match c {
            '(' | ')' | '[' | ']' | '{' | '}' | '$' | '#' | '\'' => {
                if !item.is_empty() {
                    tokens.push((item, line));
                    item = String::new();
                }
                tokens.push((c.to_string(), line));
            }
            ' ' | '\t' | '\r' => {
                if !item.is_empty() {
                    tokens.push((item, line));
                    item = String::new();
                }
            }
            '\n' => {
                if !item.is_empty() {
                    tokens.push((item, line));
                    item = String::new();
                }
                line += 1;
            }
            '"' => {
                if !item.is_empty() {
                    tokens.push((item, line));
                    item = String::new();
                }
                in_string = true;
                item.push('"');
            }
            ';' => {
                if !item.is_empty() {
                    tokens.push((item, line));
                    item = String::new();
                }
                in_comment = true;
            }
            _ => item.push(c),
        }
    }

    if in_string {
        return Err(Error::UnclosedString);
    }
    if !item.is_empty() {
        tokens.push((item, line));
    }

    Ok(tokens)
}
