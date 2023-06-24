use super::*;
use program::{Tag, TagReference};
use std::collections::HashMap;

/// The linker resolves references
pub struct Linker {
    tags: HashMap<String, Tag>,
    references: Vec<TagReference>,
    code: Vec<u8>,
}

impl Linker {
    pub fn new() -> Self {
        Self {
            tags: HashMap::new(),
            references: Vec::new(),
            code: Vec::new(),
        }
    }

    pub fn set_program(&mut self, mut program: Program) {
        self.tags.extend(program.tags.into_iter());
        self.references.append(&mut program.references);
        self.code.append(&mut program.code);
    }

    pub fn run(mut self) -> Result<Program, String> {
        let mut new_refs = Vec::<TagReference>::new();
        for i in self.references.iter() {
            let value = match self.tags.get(&i.tag) {
                Some(x) => x.value,
                None => match self.calculate(&i.tag) {
                    Some(x) => x,
                    None => {
                        new_refs.push(i.clone());
                        continue;
                    }
                },
            };
            match i.size {
                1 => {
                    self.code[i.location as usize] = value as u8;
                }
                2 => {
                    self.code[i.location as usize] = (value & 0xFF) as u8;
                    self.code[i.location as usize + 1] = (value >> 8) as u8;
                }
                x => return Err(format!("invalid tag size of {}", x)),
            }
        }
        let new_tags = self.tags.into_iter().collect::<HashMap<_, _>>();
        Ok(Program::from(self.code, new_tags, new_refs))
    }

    fn calculate(&self, expression: &str) -> Option<u16> {
        let mut split = vec![String::new()];
        for i in expression.chars() {
            if i.is_alphanumeric() || i == '_' || i == '@' {
                split.last_mut().unwrap().push(i);
            } else {
                if split.last().unwrap().len() != 0 {
                    split.push(String::new());
                }
                if !i.is_whitespace() {
                    split.last_mut().unwrap().push(i);
                    split.push(String::new());
                }
            }
        }
        if split.last().unwrap().len() == 0 {
            split.pop();
        }
        let mut split = split.iter().map(|x| x.as_str()).collect::<Vec<_>>();
        match self.parse_expression(&mut split) {
            Some(x) => Some(x as u16),
            None => None,
        }
    }

    fn parse_expression(&self, expression: &mut Vec<&str>) -> Option<i16> {
        let a = match expression.pop()? {
            ")" => self.parse_expression(expression)?,
            x => {
                if let Some(x) = util::parse_number(x) {
                    x as i16
                } else {
                    self.tags.get(x)?.value as i16
                }
            }
        };
        if let Some(x) = expression.pop() {
            match x {
                "+" => Some(self.parse_expression(expression)? + a),
                "-" => Some(self.parse_expression(expression)? - a),
                "*" => Some(self.parse_expression(expression)? * a),
                "/" => Some(self.parse_expression(expression)? / a),
                "(" => Some(a),
                _ => None,
            }
        } else {
            Some(a)
        }
    }
}
