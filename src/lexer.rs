pub fn scan(source: &String) -> Vec<(String, usize)> {
    let mut tokens = Vec::<(String, usize)>::new();
    for (line_number, line) in source.lines().enumerate() {
        let expanded_line = line.replace('(', " ( ").replace(')', " ) ").replace("$", " $ ");
        
        for token in expanded_line.split_ascii_whitespace() {
            tokens.push((token.to_string(), line_number));
        }
    }
    tokens
}