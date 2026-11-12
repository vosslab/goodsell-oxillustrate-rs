//! Focused CIF 1.1 tokenization for identity and assembly categories.
use std::collections::BTreeMap;

use crate::atom_table::Categories;
use crate::error::Error;

struct Token {
    text: String,
    bare: bool,
    line: usize,
}

fn error(line: usize, message: impl Into<String>) -> Error {
    Error::Parse {
        format: "mmCIF",
        message: format!("line {line}: {}", message.into()),
    }
}

fn tokens(input: &str) -> Result<Vec<Token>, Error> {
    let bytes = input.as_bytes();
    let mut result = Vec::new();
    let (mut i, mut line) = (0, 1);
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            if bytes[i] == b'\n' {
                line += 1;
            }
            i += 1;
            continue;
        }
        if bytes[i] == b'#' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        let start_line = line;
        let start = i;
        let (text, bare) = if bytes[i] == b';' && (i == 0 || bytes[i - 1] == b'\n') {
            i += 1;
            let content = i;
            while i < bytes.len() && !(bytes[i] == b';' && bytes[i - 1] == b'\n') {
                if bytes[i] == b'\n' {
                    line += 1;
                }
                i += 1;
            }
            if i == bytes.len() {
                return Err(error(start_line, "unterminated semicolon text field"));
            }
            let value = input[content..i].trim_matches(['\r', '\n']).to_owned();
            i += 1;
            (value, false)
        } else if matches!(bytes[i], b'\'' | b'"') {
            let quote = bytes[i];
            i += 1;
            let content = i;
            while i < bytes.len()
                && !(bytes[i] == quote
                    && (i + 1 == bytes.len() || bytes[i + 1].is_ascii_whitespace()))
            {
                if bytes[i] == b'\n' {
                    return Err(error(start_line, "newline in quoted field"));
                }
                i += 1;
            }
            if i == bytes.len() {
                return Err(error(start_line, "unterminated quoted field"));
            }
            let value = input[content..i].to_owned();
            i += 1;
            (value, false)
        } else {
            while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            (input[start..i].to_owned(), true)
        };
        result.push(Token {
            text,
            bare,
            line: start_line,
        });
    }
    Ok(result)
}

fn control(token: &Token) -> bool {
    token.bare
        && (token.text.starts_with('_')
            || token.text.starts_with("data_")
            || matches!(token.text.as_str(), "loop_" | "stop_" | "global_")
            || token.text.starts_with("save_"))
}

fn tag(token: &Token) -> Result<(&str, &str), Error> {
    token
        .text
        .strip_prefix('_')
        .and_then(|s| s.split_once('.'))
        .ok_or_else(|| {
            error(
                token.line,
                format!("expected category.field tag, got {:?}", token.text),
            )
        })
}

pub(crate) fn categories(input: &str) -> Result<Categories, Error> {
    let tokens = tokens(input)?;
    let mut result = Categories::new();
    let (mut i, mut blocks) = (0, 0);
    while i < tokens.len() {
        let token = &tokens[i];
        if token.bare && token.text.starts_with("data_") {
            blocks += 1;
            if blocks > 1 {
                return Err(error(
                    token.line,
                    "multiple data blocks are not supported; provide one structure per input",
                ));
            }
            i += 1;
        } else if token.bare && token.text == "loop_" {
            i += 1;
            let mut headers = Vec::new();
            while i < tokens.len() && tokens[i].bare && tokens[i].text.starts_with('_') {
                headers.push(tag(&tokens[i])?);
                i += 1;
            }
            if headers.is_empty() {
                return Err(error(token.line, "loop_ has no column tags"));
            }
            while i < tokens.len() && !control(&tokens[i]) {
                let start = i;
                let mut row = BTreeMap::new();
                for (category, field) in &headers {
                    if i == tokens.len() || control(&tokens[i]) {
                        return Err(error(tokens[start].line, "incomplete loop row"));
                    }
                    if *category != headers[0].0 {
                        return Err(error(tokens[start].line, "loop mixes categories"));
                    }
                    if row
                        .insert((*field).to_owned(), tokens[i].text.clone())
                        .is_some()
                    {
                        return Err(error(tokens[start].line, "duplicate loop column"));
                    }
                    i += 1;
                }
                result.entry(headers[0].0.to_owned()).or_default().push(row);
            }
        } else if token.bare && token.text.starts_with('_') {
            let (category, field) = tag(token)?;
            i += 1;
            if i == tokens.len() || control(&tokens[i]) {
                return Err(error(token.line, "tag has no value"));
            }
            let rows = result.entry(category.to_owned()).or_default();
            if rows.is_empty() {
                rows.push(BTreeMap::new());
            }
            if rows[0]
                .insert(field.to_owned(), tokens[i].text.clone())
                .is_some()
            {
                return Err(error(token.line, "duplicate category field"));
            }
            i += 1;
        } else if token.bare && token.text == "stop_" {
            i += 1;
        } else {
            return Err(error(
                token.line,
                format!("unexpected token {:?}", token.text),
            ));
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quoted_and_multiline_values_and_truncated_rows() {
        let parsed = categories("data_t\n_test.name 'a b'\n_test.description\n;first\nsecond\n;\nloop_\n_atom_site.id\n_atom_site.auth_atom_id\n1 \"C'1\"\n").unwrap();
        assert_eq!(parsed["test"][0]["description"], "first\nsecond");
        assert_eq!(parsed["atom_site"][0]["auth_atom_id"], "C'1");
        assert!(categories("data_t\nloop_\n_atom_site.id\n_atom_site.name\n1\n").is_err());
    }
}
