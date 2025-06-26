use serde::Serialize;

/// Represents a single extracted code block with its metadata and line numbers.
///
/// The lang, path, and mode fields are optional and will only be
/// included in the serialized output if they are present in the source text.
/// start_line and end_line represent the line numbers of the code content
/// (excluding the fences).
#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct CodeBlock {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    pub code: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// Extracts all code blocks from a given string slice.
///
/// This function processes the input text line by line to identify code
/// blocks that start with triple backticks (
/// It parses the info string for language, path, and mode metadata.
pub fn extract_code_blocks(text: &str) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let mut lines = text.lines();
    let mut line_number = 0;

    while let Some(line) = lines.next() {
        line_number += 1;

        if !line.starts_with("```") {
            continue;
        }

        let opening_fence = line.chars().take_while(|c| *c == '`').collect::<String>();
        let backticks = opening_fence.len();
        if backticks < 3 {
            continue;
        }

        let info_string = line[backticks..].trim();
        let mut lang = None;
        let mut path = None;
        let mut mode = None;

        let parts: Vec<&str> = info_string.split_whitespace().collect();
        if let Some(lang_part) = parts.get(0) {
            if !lang_part.is_empty() {
                lang = Some(lang_part.to_string());
            }
        }

        for part in parts.iter().skip(1) {
            if let Some(p) = part.strip_prefix("path=") {
                path = Some(p.to_string());
            } else if let Some(m) = part.strip_prefix("mode=") {
                mode = Some(m.to_string());
            }
        }

        let mut code_lines = Vec::new();
        let block_start_line = line_number + 1;
        let mut found_closing_fence = false;

        while let Some(code_line) = lines.next() {
            line_number += 1;

            let closing_fence_str = code_line
                .chars()
                .take_while(|c| *c == '`')
                .collect::<String>();
            if closing_fence_str == opening_fence
                && code_line[closing_fence_str.len()..].trim().is_empty()
            {
                found_closing_fence = true;
                break;
            }
            code_lines.push(code_line);
        }

        if found_closing_fence {
            let code = code_lines.join("\n");
            let start_line: usize = block_start_line;
            let end_line = if code_lines.is_empty() {
                start_line.saturating_sub(1)
            } else {
                start_line + code_lines.len() - 1
            };

            blocks.push(CodeBlock {
                lang,
                path,
                mode,
                code,
                start_line,
                end_line,
            });
        }
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        assert!(extract_code_blocks("").is_empty());
    }

    #[test]
    fn test_no_code_blocks() {
        let text = "This is some text without any code blocks.";
        assert!(extract_code_blocks(text).is_empty());
    }

    #[test]
    fn test_simple_code_block() {
        let text = "```rust\nlet x = 5;\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0],
            CodeBlock {
                lang: Some("rust".to_string()),
                path: None,
                mode: None,
                code: "let x = 5;".to_string(),
                start_line: 2,
                end_line: 2,
            }
        );
    }

    #[test]
    fn test_full_metadata() {
        let text = "```rust path=/tmp/foo.rs mode=EDIT\nlet x = 5;\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0],
            CodeBlock {
                lang: Some("rust".to_string()),
                path: Some("/tmp/foo.rs".to_string()),
                mode: Some("EDIT".to_string()),
                code: "let x = 5;".to_string(),
                start_line: 2,
                end_line: 2,
            }
        );
    }

    #[test]
    fn test_multiline_block() {
        let text = "```\nline 1\nline 2\nline 3\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].code, "line 1\nline 2\nline 3");
        assert_eq!(blocks[0].start_line, 2);
        assert_eq!(blocks[0].end_line, 4);
    }

    #[test]
    fn test_multiple_blocks() {
        let text = "```rust\nlet x = 5;\n```\n```python\nprint(\"hello\")\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].lang, Some("rust".to_string()));
        assert_eq!(blocks[0].code, "let x = 5;");
        assert_eq!(blocks[0].start_line, 2);
        assert_eq!(blocks[0].end_line, 2);
        assert_eq!(blocks[1].lang, Some("python".to_string()));
        assert_eq!(blocks[1].code, "print(\"hello\")");
        assert_eq!(blocks[1].start_line, 5);
        assert_eq!(blocks[1].end_line, 5);
    }

    #[test]
    fn test_empty_block() {
        let text = "```rust\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].code, "");
        assert_eq!(blocks[0].start_line, 2);
        assert_eq!(blocks[0].end_line, 1); // Special case: empty range
    }

    #[test]
    fn test_extended_fence() {
        let text = "````rust\nlet x = 5;\n````";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].lang, Some("rust".to_string()));
        assert_eq!(blocks[0].code, "let x = 5;");
        assert_eq!(blocks[0].start_line, 2);
        assert_eq!(blocks[0].end_line, 2);
    }

    #[test]
    fn test_nested_fences() {
        let text = "```\n```rust\nlet x = 5;\n```\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].code, "```rust\nlet x = 5;");
        assert_eq!(blocks[0].start_line, 2);
        assert_eq!(blocks[0].end_line, 3);
    }

    #[test]
    fn test_unclosed_block() {
        let text = "```rust\nlet x = 5;";
        let blocks = extract_code_blocks(text);
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_closing_fence_with_whitespace() {
        let text = "```rust\nlet x = 5;\n```   ";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].code, "let x = 5;");
        assert_eq!(blocks[0].start_line, 2);
        assert_eq!(blocks[0].end_line, 2);
    }

    #[test]
    fn test_closing_fence_min_length() {
        let text = "```rust\nlet x = 5;\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].code, "let x = 5;");
        assert_eq!(blocks[0].start_line, 2);
        assert_eq!(blocks[0].end_line, 2);
    }

    #[test]
    fn test_indented_closing_fence_ignored() {
        let text = "```rust\nlet x = 5;\n  ```";
        let blocks = extract_code_blocks(text);
        assert!(blocks.is_empty());
    }
}
