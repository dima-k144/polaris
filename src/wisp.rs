use crate::types::Error;

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
    if line.trim().is_empty() {
        0
    } else {
        match indent_type {
            Indentation::TwoSpaces => (line.len() - line.trim_start_matches(' ').len()) / 2,
            Indentation::FourSpaces => (line.len() - line.trim_start_matches(' ').len()) / 4,
            Indentation::Tabs => line.len() - line.trim_start_matches('\t').len(),
        }
    }
}

pub fn preprocess_wisp3(input: &str) -> Result<String, Error> {
    let indent_type = detect_indentation(input)?;

    let mut lines: Vec<String> = input.lines().map(String::from).collect();
    lines.push("".to_string());
    let mut i = 0;

    for i in 0..lines.len() {
        lines[i] = strip_comments(&lines[i]).to_string()
    }

    while i != lines.len() {
        // empty line
        if lines[i].trim().is_empty() {
            i += 1;
            continue;
        }

        // traditional s exp
        if lines[i].trim().starts_with('(') || lines[i].trim().starts_with("{") {
            while !lines[i].trim().is_empty() {
                i += 1;
            }
            continue;
        }

        // add (
        let cur_indent = indent(&lines[i], &indent_type);
        match indent_type {
            Indentation::TwoSpaces => lines[i].insert(cur_indent * 2, '('),
            Indentation::FourSpaces => lines[i].insert(cur_indent * 4, '('),
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
            for _ in 0..diff {
                lines[i].push(')');
            }
        }

        i += 1;
    }

    Ok(lines.join("\n"))
}
