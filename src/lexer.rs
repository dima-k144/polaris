use crate::types::Error;

pub fn scan(source: &String) -> Result<Vec<(String, usize)>, Error> {
    let mut tokens = Vec::<(String, usize)>::new();
    let mut item = String::new();
    let mut line = 0;
    let mut in_string = false;
    for c in source.chars() {
        if in_string {
            match c {
                '"' => {
                    in_string = false;
                    item.push('"');
                    tokens.push((item, line)); item = String::new();
                },
                c => {
                    item.push(c)
                }
            }
        } else {
            match c {
                '(' | ')' | '$' => {
                    if !item.is_empty() {
                        tokens.push((item, line));
                        item = String::new();
                    }
                    tokens.push((c.to_string(), line));
                }
                ' ' | '\t' | '\r' => { 
                    if !item.is_empty() { tokens.push((item, line)); item = String::new(); } 
                },
                '\n' => { 
                    if !item.is_empty() { tokens.push((item, line)); item = String::new(); } 
                    line += 1; 
                },
                '"' => {
                    if !item.is_empty() { tokens.push((item, line)); item = String::new(); } 
                    in_string = true;
                    item.push('"');
                }
                c => { item.push(c); }
        }
        }

    }

    if in_string {
        return Err(Error::UnclosedString);
    }

    if !item.is_empty() { tokens.push((item, line)) };
    Ok(tokens)
}
