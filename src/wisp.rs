use crate::types::Error;

#[derive(Debug, Clone)]
struct LogicalLine {
    line_number: usize,
    indent: usize,
    prefix: String,
    text: String,
    continuation: bool,
    traditional: bool,
}

#[derive(Debug, Clone, Copy)]
struct DelimiterState {
    paren: usize,
    bracket: usize,
    brace: usize,
    in_string: bool,
    escaped: bool,
}

impl DelimiterState {
    fn new() -> Self {
        Self {
            paren: 0,
            bracket: 0,
            brace: 0,
            in_string: false,
            escaped: false,
        }
    }

    fn depth(&self) -> usize {
        self.paren + self.bracket + self.brace
    }

    fn balanced(&self) -> bool {
        self.depth() == 0 && !self.in_string
    }
}

/// Remove a semicolon comment while preserving semicolons inside strings.
fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    let mut escaped = false;

    for (index, ch) in line.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else {
            match ch {
                '"' => in_string = true,
                ';' => return &line[..index],
                _ => {}
            }
        }
    }

    line
}

/// Count indentation columns.
///
/// Spaces count as one column and tabs count as four columns. Wisp levels
/// are relative, so this does not impose a fixed indentation width.
fn indentation_width(line: &str) -> usize {
    let mut width = 0;

    for ch in line.chars() {
        match ch {
            ' ' => width += 1,
            '\t' => width += 4,
            _ => break,
        }
    }

    width
}

/// Return the original indentation prefix.
fn leading_whitespace(line: &str) -> String {
    line.chars()
        .take_while(|ch| *ch == ' ' || *ch == '\t')
        .collect()
}

/// Convert `\:` into a literal colon.
///
/// Underscore indentation is intentionally not implemented.
fn unescape_colons(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' && chars.peek() == Some(&':') {
            chars.next();
            result.push(':');
        } else {
            result.push(ch);
        }
    }

    result
}

/// Scan delimiters in one physical line.
///
/// This is used for traditional Scheme expressions so that line breaks
/// inside `()`, `[]`, `{}`, and strings are not interpreted as Wisp lines.
fn scan_delimiters(
    line: &str,
    state: &mut DelimiterState,
) -> Result<(), Error> {
    let mut in_comment = false;

    for ch in line.chars() {
        if in_comment {
            continue;
        }

        if state.in_string {
            if state.escaped {
                state.escaped = false;
            } else if ch == '\\' {
                state.escaped = true;
            } else if ch == '"' {
                state.in_string = false;
            }

            continue;
        }

        match ch {
            ';' => {
                in_comment = true;
            }

            '"' => {
                state.in_string = true;
            }

            '(' => {
                state.paren += 1;
            }

            ')' => {
                if state.paren == 0 {
                    return Err(Error::UnexpectedClosedParen);
                }

                state.paren -= 1;
            }

            '[' => {
                state.bracket += 1;
            }

            ']' => {
                if state.bracket == 0 {
                    return Err(Error::Custom(
                        "unexpected ']' while preprocessing Wisp".into(),
                    ));
                }

                state.bracket -= 1;
            }

            '{' => {
                state.brace += 1;
            }

            '}' => {
                if state.brace == 0 {
                    return Err(Error::Custom(
                        "unexpected '}' while preprocessing Wisp".into(),
                    ));
                }

                state.brace -= 1;
            }

            _ => {}
        }
    }

    Ok(())
}

/// A line beginning with `(`, `[`, or `{` starts traditional Lisp syntax.
fn starts_traditional_expression(text: &str) -> bool {
    matches!(
        text.chars().next(),
        Some('(') | Some('[') | Some('{')
    )
}

/// Replace Wisp inline colons.
///
/// Examples:
///
/// ```text
/// a : b
/// ```
///
/// becomes:
///
/// ```text
/// a (b)
/// ```
///
/// Colons inside strings and ordinary parenthesized expressions are ignored.
fn transform_inline_colons(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();

    let mut output = String::new();
    let mut fragment = String::new();

    let mut in_string = false;
    let mut escaped = false;
    let mut delimiter_depth = 0usize;
    let mut inline_colons = 0usize;
    let mut skip_whitespace_after_colon = false;

    for index in 0..chars.len() {
        let ch = chars[index];

        if in_string {
            fragment.push(ch);

            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }

            continue;
        }

        /*
         * Do not preserve the whitespace after an inline colon:
         *
         *     a : b
         *
         * should become:
         *
         *     a (b)
         */
        if skip_whitespace_after_colon && ch.is_whitespace() {
            continue;
        }

        skip_whitespace_after_colon = false;

        match ch {
            '"' => {
                in_string = true;
                fragment.push(ch);
            }

            '(' | '[' | '{' => {
                delimiter_depth += 1;
                fragment.push(ch);
            }

            ')' | ']' | '}' => {
                delimiter_depth = delimiter_depth.saturating_sub(1);
                fragment.push(ch);
            }

            ':' if delimiter_depth == 0 => {
                let before_is_whitespace =
                    index == 0 || chars[index - 1].is_whitespace();

                let after_is_whitespace =
                    index + 1 == chars.len() || chars[index + 1].is_whitespace();

                if before_is_whitespace && after_is_whitespace {
                    /*
                     * Remove whitespace before the colon.
                     */
                    while fragment.ends_with(char::is_whitespace) {
                        fragment.pop();
                    }

                    output.push_str(&fragment);
                    fragment.clear();

                    /*
                     * Add exactly one separator before the generated list.
                     *
                     * This is the part that fixes:
                     *
                     *     a(b)
                     *
                     * into:
                     *
                     *     a (b)
                     */
                    if !output.is_empty()
                        && !output.ends_with(char::is_whitespace)
                    {
                        output.push(' ');
                    }

                    output.push('(');
                    inline_colons += 1;
                    skip_whitespace_after_colon = true;
                } else {
                    fragment.push(ch);
                }
            }

            _ => {
                fragment.push(ch);
            }
        }
    }

    output.push_str(&fragment);

    for _ in 0..inline_colons {
        output.push(')');
    }

    output
}

/// Parse one non-empty physical line.
fn parse_line(line_number: usize, raw_line: &str) -> Option<LogicalLine> {
    let uncommented = strip_comment(raw_line);
    let trimmed = uncommented.trim();

    if trimmed.is_empty() {
        return None;
    }

    let indent = indentation_width(uncommented);
    let prefix = leading_whitespace(uncommented);
    let text = unescape_colons(trimmed);

    let continuation = text == "." || text.starts_with(". ");
    let traditional = starts_traditional_expression(&text);

    Some(LogicalLine {
        line_number,
        indent,
        prefix,
        text,
        continuation,
        traditional,
    })
}

/// Convert Wisp indentation into parenthesized Lisp.
///
/// Traditional S-expressions are copied unchanged, including their internal
/// indentation. Wisp lines receive generated parentheses while preserving
/// their source indentation.
pub fn preprocess_wisp3(input: &str) -> Result<String, Error> {
    let source_lines: Vec<&str> = input.lines().collect();

    /*
     * First create logical lines. Traditional parenthesized expressions are
     * collected as complete blocks so that their internal lines are never
     * interpreted as Wisp syntax.
     */
    let mut logical_lines = Vec::<LogicalLine>::new();
    let mut source_index = 0usize;

    while source_index < source_lines.len() {
        let raw_line = source_lines[source_index];

        let Some(parsed) = parse_line(source_index + 1, raw_line) else {
            source_index += 1;
            continue;
        };

        if parsed.traditional {
            let start_line_number = parsed.line_number;
            let mut block = String::new();
            let mut delimiter_state = DelimiterState::new();

            loop {
                if source_index >= source_lines.len() {
                    return Err(Error::UnclosedListExpr);
                }

                let block_line = source_lines[source_index];

                if !block.is_empty() {
                    block.push('\n');
                }

                block.push_str(block_line);

                scan_delimiters(block_line, &mut delimiter_state)?;
                source_index += 1;

                if delimiter_state.balanced() {
                    break;
                }
            }

            logical_lines.push(LogicalLine {
                line_number: start_line_number,
                indent: 0,
                prefix: String::new(),
                text: block,
                continuation: false,
                traditional: true,
            });

            continue;
        }

        logical_lines.push(parsed);
        source_index += 1;
    }

    let mut output_lines: Vec<String> = Vec::new();

    /*
     * Each entry represents a generated Wisp list that is currently open.
     * Closing parentheses are appended to the previous output line.
     */
    let mut open_indents: Vec<usize> = Vec::new();

    for index in 0..logical_lines.len() {
        let current = &logical_lines[index];

        let next_indent = logical_lines
            .get(index + 1)
            .map(|line| line.indent)
            .unwrap_or(0);

        /*
         * Traditional Lisp is emitted byte-for-byte as a logical block.
         */
        if current.traditional {
            output_lines.push(format!(
                "{}{}",
                current.prefix, current.text
            ));

            continue;
        }

        /*
         * A Wisp continuation line does not open a generated list.
         */
        if current.continuation {
            let body = current
                .text
                .strip_prefix('.')
                .unwrap_or("")
                .trim_start();

            if body.is_empty() {
                return Err(Error::Custom(format!(
                    "Wisp syntax error on line {}: continuation requires an expression after '.'",
                    current.line_number
                )));
            }

            output_lines.push(format!(
                "{}(. {})",
                current.prefix, body
            ));

            continue;
        }

        /*
         * Close generated lists before emitting a line at the same or lower
         * indentation. Parentheses are attached to the preceding line.
         */
        while let Some(open_indent) = open_indents.last().copied() {
            if current.indent <= open_indent {
                let Some(previous) = output_lines.last_mut() else {
                    return Err(Error::Custom(
                        "internal Wisp error: missing previous output line".into(),
                    ));
                };

                previous.push(')');
                open_indents.pop();
            } else {
                break;
            }
        }

        /*
         * A dedent must target an already established indentation level.
         */
        if let Some(parent_indent) = open_indents.last().copied() {
            if current.indent < parent_indent {
                return Err(Error::Custom(format!(
                    "Wisp syntax error on line {}: indentation {} does not match an existing indentation level",
                    current.line_number,
                    current.indent
                )));
            }
        }

        let transformed = transform_inline_colons(&current.text);

        /*
         * A lone colon creates an empty grouping list.
         */
        if transformed == ":" {
            output_lines.push(format!("{}(", current.prefix));
            open_indents.push(current.indent);
            continue;
        }

        /*
         * A normal Wisp line becomes one list.
         */
        output_lines.push(format!(
            "{}({}",
            current.prefix, transformed
        ));

        open_indents.push(current.indent);

        /*
         * If the next logical line is not more indented, this list closes on
         * the same output line.
         */
        if next_indent <= current.indent {
            let Some(previous) = output_lines.last_mut() else {
                return Err(Error::Custom(
                    "internal Wisp error: missing generated line".into(),
                ));
            };

            previous.push(')');
            open_indents.pop();
        }
    }

    /*
     * Close any remaining generated lists at end-of-input.
     */
    while open_indents.pop().is_some() {
        let Some(last) = output_lines.last_mut() else {
            return Err(Error::Custom(
                "internal Wisp error: no output line to close".into(),
            ));
        };

        last.push(')');
    }

    /*
     * Preserve the leading and trailing newline behavior used by the tests.
     */
    let mut result = String::new();

    if input.starts_with('\n') {
        result.push('\n');
    }

    for (index, line) in output_lines.iter().enumerate() {
        result.push_str(line);

        if index + 1 < output_lines.len() || input.ends_with('\n') {
            result.push('\n');
        }
    }

    Ok(result)
}