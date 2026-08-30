use crate::types::Error;

pub fn preprocess_wisp_____(input: &str) -> Result<String, Error> {
    let mut result = String::new();
    let mut indent_stack: Vec<usize> = Vec::new();
    let mut paren_depth: usize = 0;
    let mut bracket_depth: usize = 0;
    let mut brace_depth: usize = 0;
    let mut in_string = false;
    let mut consecutive_empty_lines = 0;

    for raw_line in input.lines() {
        let line_without_comment = strip_comments(raw_line);
        let trimmed = line_without_comment.trim();

        if trimmed.is_empty() {
            consecutive_empty_lines += 1;
            if consecutive_empty_lines >= 2 {
                while indent_stack.pop().is_some() {
                    result.push_str(") ");
                }
            }
            result.push('\n');
            continue;
        }
        consecutive_empty_lines = 0;

        // If inside an open paren/bracket/brace or multiline string, pass through directly
        if in_string || paren_depth > 0 || bracket_depth > 0 || brace_depth > 0 {
            update_delimiters(
                trimmed,
                &mut in_string,
                &mut paren_depth,
                &mut bracket_depth,
                &mut brace_depth,
            );
            result.push_str(trimmed);
            result.push('\n');
            continue;
        }

        // Standard parenthesized S-expression at top-level: bypass indent stack
        if trimmed.starts_with('(') || trimmed.starts_with('[') || trimmed.starts_with('{') {
            while indent_stack.pop().is_some() {
                result.push_str(") ");
            }
            update_delimiters(
                trimmed,
                &mut in_string,
                &mut paren_depth,
                &mut bracket_depth,
                &mut brace_depth,
            );
            result.push_str(trimmed);
            result.push('\n');
            continue;
        }

        let indent = raw_line
            .chars()
            .take_while(|c| *c == ' ' || *c == '\t')
            .map(|c| if c == '\t' { 4 } else { 1 })
            .sum::<usize>();

        let is_dot_continuation = trimmed == "." || trimmed.starts_with(". ");
        let is_colon_only = trimmed == ":";

        while let Some(&top) = indent_stack.last() {
            if indent < top {
                indent_stack.pop();
                result.push_str(") ");
            } else {
                break;
            }
        }

        let content_to_process = if is_dot_continuation {
            if trimmed == "." { "" } else { &trimmed[2..] }
        } else if is_colon_only {
            ""
        } else {
            trimmed
        };

        if is_dot_continuation {
            // A "." line continues the *enclosing* form. The previous sibling
            // at this indent (e.g. `cast …`) auto-opened a "(" — close it (and
            // anything else at this level) so the content lands in the parent.
            while let Some(&top) = indent_stack.last() {
                if top >= indent {
                    indent_stack.pop();
                    result.push_str(") ");
                } else {
                    break;
                }
            }
            let expanded = expand_inline_colons(content_to_process);
            result.push_str(&expanded);
        } else if is_colon_only {
            indent_stack.push(indent);
            result.push_str("(");
        } else {
            if indent_stack.last().map_or(true, |&top| indent > top) {
                indent_stack.push(indent);
                result.push_str("(");
            } else {
                result.push_str(") (");
            }
            let expanded = expand_inline_colons(content_to_process);
            result.push_str(&expanded);
        }

        update_delimiters(
            content_to_process,
            &mut in_string,
            &mut paren_depth,
            &mut bracket_depth,
            &mut brace_depth,
        );
        result.push('\n');
    }

    while indent_stack.pop().is_some() {
        result.push_str(") ");
    }

    Ok(result)
}

fn strip_comments(s: &str) -> &str {
    let mut in_str = false;
    let mut escape = false;
    for (i, c) in s.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if c == '\\' {
            escape = true;
        } else if c == '"' {
            in_str = !in_str;
        } else if c == ';' && !in_str {
            return &s[..i];
        }
    }
    s
}

fn update_delimiters(
    s: &str,
    in_string: &mut bool,
    paren_depth: &mut usize,
    bracket_depth: &mut usize,
    brace_depth: &mut usize,
) {
    let mut escape = false;
    for c in s.chars() {
        if escape {
            escape = false;
            continue;
        }
        if c == '\\' {
            escape = true;
            continue;
        }
        if c == '"' {
            *in_string = !*in_string;
        } else if !*in_string {
            match c {
                '(' => *paren_depth += 1,
                ')' => *paren_depth = paren_depth.saturating_sub(1),
                '[' => *bracket_depth += 1,
                ']' => *bracket_depth = bracket_depth.saturating_sub(1),
                '{' => *brace_depth += 1,
                '}' => *brace_depth = brace_depth.saturating_sub(1),
                _ => {}
            }
        }
    }
}

fn expand_inline_colons(line: &str) -> String {
    let mut trimmed = line.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut prefix_parens = 0;
    if trimmed.starts_with(": ") {
        prefix_parens += 1;
        trimmed = &trimmed[2..];
    } else if trimmed == ":" {
        return "()".to_string();
    }

    let mut ends_with_empty_colon = false;
    if trimmed.ends_with(" :") {
        ends_with_empty_colon = true;
        trimmed = &trimmed[..trimmed.len() - 2];
    }

    let parts: Vec<&str> = trimmed.split(" : ").collect();
    let inline_colons = parts.len().saturating_sub(1);

    let mut out = "(".repeat(prefix_parens);
    out.push_str(parts[0]);

    for part in &parts[1..] {
        out.push_str(" (");
        out.push_str(part);
    }

    if ends_with_empty_colon {
        out.push_str(" ()");
    }

    out.push_str(&")".repeat(inline_colons + prefix_parens));
    out.replace("\\:", ":")
}

fn count_indent_level(s: &String) -> usize {
    s.chars().take_while(|&c| c == ' ').count()
}

fn parent(i: usize, lines: &Vec::<String>) -> Option<usize> {
    for j in i..0 {
        if count_indent_level(&lines[j]) == count_indent_level(&lines[i]) {
            return Some(j)
        }
    }
    None
}

pub fn preprocess_wisp2_____(input: &str) -> Result<String, Error> {
    let mut lines: Vec::<String> = input.lines().map(String::from).collect();
    lines.insert(0, "".to_string());
    lines.push("".to_string());

    let mut unclosed_parens = 0;
    for i in 1..lines.len() {
        let indent_level = count_indent_level(&lines[i]);
        println!("{}", indent_level);

        if lines[i].trim().starts_with('(') || lines[i - 1].trim().starts_with('(') {
            continue;
        }

        if lines[i].trim().is_empty() {
            for _ in 0..unclosed_parens {
                lines[i-1].push(')');
            }
            unclosed_parens = 0;
        }

        if !lines[i].trim().is_empty() {
            lines[i].insert(indent_level, '(');
            lines[i].push(')');
        }

        if count_indent_level(&lines[i-1]) < indent_level {
            lines[i-1].pop();
            unclosed_parens += 1;
        }

        if count_indent_level(&lines[i-1]) > indent_level {
            lines[i-1].push(')');
        }
    }

    Ok(lines.join("\n"))
}

#[derive(Debug, PartialEq, Eq)]
pub enum Indentation {
    TwoSpaces,
    FourSpaces,
    Tabs,
}

pub fn detect_indentation(source: &str) -> Result<Indentation, Error> {
    let mut detected: Option<Indentation> = None;
    let mut saw_odd_2_space_multiple = false; // Tracks if we ever saw 2, 6, 10 spaces

    for (line_idx, line) in source.lines().enumerate() {
        let line_num = line_idx + 1;

        // Skip completely empty lines or whitespace-only lines
        if line.trim().is_empty() {
            continue;
        }

        let mut space_count = 0;
        let mut tab_count = 0;

        for ch in line.chars() {
            match ch {
                ' ' => space_count += 1,
                '\t' => tab_count += 1,
                _ => break, // Hit first non-whitespace character
            }
        }

        // Line is not indented
        if space_count == 0 && tab_count == 0 {
            continue;
        }

        // Error: mixing spaces and tabs on the very same line
        if space_count > 0 && tab_count > 0 {
            return Err(Error::MixedWithinLine(line_num));
        }

        // 1. Tab indentation branch
        if tab_count > 0 {
            match detected {
                Some(Indentation::Tabs) => continue,
                Some(_) => return Err(Error::MixedAcrossFile(line_num)),
                None => detected = Some(Indentation::Tabs),
            }
        }

        // 2. Space indentation branch
        if space_count > 0 {
            if space_count % 2 != 0 {
                return Err(Error::AmbiguousSpaces(line_num));
            }

            match detected {
                Some(Indentation::Tabs) => {
                    return Err(Error::MixedAcrossFile(line_num));
                }
                Some(Indentation::FourSpaces) => {
                    if space_count % 4 != 0 {
                        // A line with 2, 6, 10 spaces conflicts with a confirmed 4-space file
                        return Err(Error::MixedAcrossFile(line_num));
                    }
                }
                Some(Indentation::TwoSpaces) => {
                    // Already confirmed 2 spaces, any multiple of 2 is valid
                    continue;
                }
                None => {
                    // If divisible by 4, it could still be either (e.g. 4 spaces can mean 2 levels of 2-spaces)
                    if space_count % 4 == 0 {
                        // Mark tentatively, will be refined if we hit 2 or 6 spaces later
                        detected = Some(Indentation::FourSpaces);
                    } else {
                        // e.g. 2, 6, 10 spaces -> definitely 2-space indentation
                        detected = Some(Indentation::TwoSpaces);
                        saw_odd_2_space_multiple = true;
                    }
                }
            }

            // If we tentatively guessed FourSpaces but later hit an odd multiple of 2 (like 2 or 6 spaces)
            if detected == Some(Indentation::FourSpaces) && space_count % 4 != 0 {
                detected = Some(Indentation::TwoSpaces);
                saw_odd_2_space_multiple = true;
            }
        }
    }

    match detected {
        Some(value) => Ok(value),
        None => Ok(Indentation::FourSpaces),
    }
}

fn indent(line: &str, indent_type: &Indentation) -> usize {
    if line.trim().is_empty() { 0 }
    else { 
        match indent_type {
            Indentation::TwoSpaces => (line.len() - line.trim_start_matches(' ').len()) / 2,
            Indentation::FourSpaces => (line.len() - line.trim_start_matches(' ').len()) / 4,
            Indentation::Tabs => line.len() - line.trim_start_matches('\t').len(),
        }
    }
}

pub fn preprocess_wisp3(input: &str) -> Result<String, Error> {
    let indent_type = detect_indentation(input)?;
    
    let mut lines: Vec::<String> = input.lines().map(String::from).collect();
    lines.push("".to_string());
    let mut i = 0;

    for i in 0..lines.len() {
        lines[i] = strip_comments(&lines[i]).to_string()
    }

    while i != lines.len() {
        // empty line
        if lines[i].trim().is_empty() { i += 1; continue; }

        // traditional s exp
        if lines[i].trim().starts_with('(') {
            while !lines[i].trim().is_empty() {
                i += 1;
            }
            continue;
        }

        // add (
        let cur_indent = indent(&lines[i], &indent_type);
        match indent_type {
            Indentation::TwoSpaces => lines[i].insert(cur_indent*2, '('),
            Indentation::FourSpaces => lines[i].insert(cur_indent*4, '('),
            Indentation::Tabs => lines[i].insert(cur_indent, '('),
        };

        // handle :
        if lines[i].contains(" : ") {
            for _ in 0..lines[i].matches(" : ").count() {
                lines[i].push(')');
            }
            lines[i] = lines[i].replace(" : ", " (");
        }

        if indent(&lines[i], &indent_type) == indent(&lines[i + 1], &indent_type) {
            lines[i].push(')');
        } else if indent(&lines[i], &indent_type) > indent(&lines[i + 1], &indent_type) {
            let diff = indent(&lines[i], &indent_type) - indent(&lines[i + 1], &indent_type);
            lines[i].push(')'); 
            for _ in 0..diff { lines[i].push(')'); }
        }

        i += 1;
    }

    Ok(lines.join("\n"))
}
