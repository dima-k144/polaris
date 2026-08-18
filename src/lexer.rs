pub fn scan(source: &String) -> Vec<(String, usize)> {
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
    if !item.is_empty() { tokens.push((item, line)) };
    tokens
}
