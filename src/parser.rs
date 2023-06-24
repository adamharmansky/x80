use std::io::prelude::*;

/// Outputs lines, split into unprocessed tokens
pub struct Parser {
    file: std::iter::Enumerate<std::io::Lines<std::io::BufReader<Box<dyn Read>>>>,
}

impl Iterator for Parser {
    type Item = (usize, Vec<String>);

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.file.next()?;
        let line_number = line.0;
        let line = line.1.unwrap();
        let line = line.trim();
        // don't proceed any further on an empty line
        if line.len() == 0 {
            return Some((line_number, Vec::new()));
        }
        let mut tokens = vec![String::new()];
        let mut bracket = false;
        let mut escape = false;
        for i in line.chars() {
            let empty = tokens.last()?.len() == 0;

            if bracket {
                if i == '}' {
                    if !escape {
                        bracket = false;
                        continue;
                    }
                }
                if i == '\\' {
                    if !escape {
                        escape = true;
                        continue;
                    } else {
                        escape = false;
                    }
                } else {
                    escape = false;
                };
                tokens.last_mut()?.push(i);
                continue;
            }

            if i == '{' {
                bracket = true;
            } else if i.is_whitespace() {
                if !empty {
                    tokens.push(String::new());
                }
            } else if !i.is_alphanumeric() && !"_@$':".contains(i) {
                if empty {
                    tokens.last_mut()?.extend(i.to_lowercase());
                    tokens.push(String::new());
                } else {
                    tokens.push(String::new());
                    tokens.last_mut()?.extend(i.to_lowercase());
                    tokens.push(String::new());
                }
            } else {
                tokens.last_mut()?.push(i);
            }
        }
        // Remove the last element if empty
        if tokens.last()?.len() == 0 {
            tokens.pop();
        }
        Some((line_number, tokens))
    }
}

impl Parser {
    pub fn new(file: Box<dyn Read>) -> Self {
        let file = std::io::BufReader::new(file);
        let file = file.lines();
        let file = file.enumerate();
        Self { file }
    }
}
