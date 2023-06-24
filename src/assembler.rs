use super::*;
use program::ProgramWriter;
use std::collections::{HashMap, HashSet};
use std::io::BufRead;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum Token {
    Keyword(String),
    Value,
}

#[derive(Clone, Debug)]
struct Opcode {
    bytes: Vec<u8>,
    /// Sizes of operands in bytes
    operands: Vec<Operand>,
}

#[derive(Clone, Debug)]
struct Operand {
    /// The size of the operand in bytes
    size: u8,
    /// The offset of the operand from the start of the instruction, in bytes
    position: u8,
}

/// Parses instructions into machine code.
pub struct Assembler {
    keywords: HashSet<String>,
    opcodes: HashMap<Vec<Token>, Opcode>,
    scopes: Vec<String>,
    program: ProgramWriter,
    anon_generator: std::ops::RangeFrom<i32>,
    allocator: u16,
}

impl Assembler {
    pub fn new() -> Self {
        const OPCODES: &str = include_str!("opcodes.txt");
        let mut me = Self {
            keywords: HashSet::new(),
            opcodes: HashMap::new(),
            scopes: Vec::new(),
            program: ProgramWriter::new(),
            anon_generator: 0..,
            allocator: 0xFFFF,
        };
        let mut file = std::io::BufReader::new(std::io::Cursor::new(OPCODES));
        me.load_instructions(&mut file);
        me
    }

    fn load_instructions(&mut self, file: &mut dyn BufRead) {
        for i in file.lines() {
            let i = i.unwrap();
            let i = i.trim();
            if i.len() == 0 {
                continue;
            }
            let mut i = i.split("->").map(|x| x.split(" "));
            let mut syntax = i
                .next()
                .expect("wrong opcode syntax!")
                .filter(|x| !x.is_empty())
                .map(|x| Token::Keyword(String::from(x)))
                .collect::<Vec<_>>();
            let opcode = i
                .next()
                .expect("wrong opcode syntax!")
                .filter(|x| !x.is_empty())
                .collect::<Vec<_>>();
            let mut operands = Vec::<Operand>::new();
            for i in &mut syntax {
                if let Token::Keyword(x) = i {
                    if x.as_str() == "[8]" {
                        *i = Token::Value;
                        operands.push(Operand {
                            size: 1,
                            position: 0,
                        });
                    } else if x.as_str() == "[16]" {
                        *i = Token::Value;
                        operands.push(Operand {
                            size: 2,
                            position: 0,
                        });
                    } else {
                        self.add_keyword(x.clone())
                    }
                }
            }
            let mut opcode_bytes = Vec::<u8>::new();
            for i in opcode.iter().enumerate() {
                if &i.1[0..1] == "[" {
                    let position = (*i.1)
                        .trim_matches(|x| x == '[' || x == ']')
                        .parse::<usize>()
                        .unwrap();
                    operands[position].position = i.0 as u8;
                    for _ in 0..operands[position].size {
                        opcode_bytes.push(0x00);
                    }
                } else {
                    opcode_bytes.push(u8::from_str_radix(*i.1, 16).unwrap());
                }
            }
            self.add_opcode(syntax, opcode_bytes, operands);
        }
    }

    fn add_keyword(&mut self, word: String) {
        self.keywords.insert(word);
    }

    fn add_opcode(&mut self, condition: Vec<Token>, bytes: Vec<u8>, operands: Vec<Operand>) {
        self.opcodes.insert(condition, Opcode { bytes, operands });
    }

    /// Parse an instruction and generate code, appending to the program
    pub fn parse(&mut self, instruction: Vec<String>) -> Result<(), String> {
        if instruction.len() == 0 {
            return Ok(());
        }
        if instruction.len() == 1 {
            if instruction[0].as_str() == "@end" {
                self.scopes
                    .pop()
                    .ok_or(String::from("@end called in global scope!"))?;
                return Ok(());
            }
            if instruction[0].as_str() == "@scope" {
                self.scopes
                    .push(format!("_anon_{}_", self.anon_generator.next().unwrap()));
                return Ok(());
            }
            if instruction[0].chars().rev().next().unwrap() == ':' {
                let label = &instruction[0][..(instruction[0].len() - 1)];
                if let Some(x) = util::parse_number(label) {
                    self.program.seek(x as u16);
                } else {
                    self.add_tag(label, self.program.pc())?;
                }
                return Ok(());
            }
        } else if instruction.len() == 2 {
            if instruction[0].as_str() == "@string" {
                for i in instruction[1].bytes() {
                    self.program.write(i);
                }
                return Ok(());
            }
            if instruction[0].as_str() == "@mod" {
                self.scopes.push(instruction[1].clone());
                return Ok(());
            }
            if instruction[0].as_str() == "@function" {
                self.add_tag(instruction[1].as_str(), self.program.pc())?;
                self.scopes.push(instruction[1].clone());
                return Ok(());
            }
            if instruction[0].as_str() == "@alloc_from" {
                self.allocator = util::parse_number(&instruction[1])
                    .ok_or(String::from("cannot parse number"))?
                    as u16;
                return Ok(());
            }
        } else if instruction.len() == 3 {
            if instruction[0].as_str() == "@define" {
                self.add_tag(
                    instruction[1].as_str(),
                    util::parse_number(&instruction[2])
                        .ok_or(String::from("cannot parse number"))? as _,
                )?;
                return Ok(());
            }
            if instruction[0].as_str() == "@alloc" {
                self.add_tag(instruction[1].as_str(), self.allocator)?;
                self.allocator += util::parse_number(&instruction[2])
                    .ok_or(String::from("cannot parse number"))?
                    as u16;
                return Ok(());
            }
        }
        if instruction.len() > 0 {
            // commments
            if instruction[0] == "@" {
                return Ok(());
            }
        }
        // parse the line to find keywords and values
        let analyzed = instruction
            .iter()
            .map(|x| self.analyze(x))
            .collect::<Vec<_>>();
        if let Some(opcode) = self.opcodes.get(&analyzed) {
            // Clone the opcode so that we don't own a reference to it
            let opcode = opcode.clone();
            for i in &opcode.bytes {
                self.program.write(*i);
            }
            let mut sizes = opcode.operands.iter();
            for i in analyzed.iter().enumerate() {
                if let Token::Value = i.1 {
                    let operand = sizes.next().unwrap();
                    let value = &instruction[i.0];
                    self.program.push_pc();
                    self.program.seek(
                        self.program.pc() - opcode.bytes.len() as u16 + operand.position as u16,
                    );
                    self.parse_value(value, operand.size)?;
                    self.program.pop_pc();
                }
            }
            Ok(())
        } else {
            Err(format!("syntax error"))
        }
    }

    /// Destroys self and gives back its program
    pub fn program(self) -> Program {
        self.program.into_program()
    }

    /// Creates a "token" out of a word
    fn analyze(&mut self, token: &String) -> Token {
        if self.keywords.contains(token) {
            Token::Keyword(token.clone())
        } else {
            Token::Value
        }
    }

    /// Tries to parse value as number, creates a tag reference if unable to do so
    fn parse_value(&mut self, value: &str, size: u8) -> Result<(), String> {
        if let Some(x) = util::parse_number(value) {
            if size == 1 {
                if let Ok(x) = u8::try_from(x) {
                    self.program.write(x);
                    Ok(())
                } else {
                    Err(format!("{} is not an 8-bit number!", x))
                }
            } else if size == 2 {
                if let Ok(x) = u16::try_from(x) {
                    self.program.write((x & 0xFF) as u8);
                    self.program.write((x >> 8) as u8);
                    Ok(())
                } else {
                    Err(format!("{} is not a 16-bit number!", x))
                }
            } else {
                panic!("only operand sizes 1 and 2 are supported!");
            }
        } else {
            match size {
                1 => {
                    self.add_reference(value, self.program.pc(), 1)?;
                    self.program.write(0x00);
                    self.program.write(0x00);
                }
                2 => {
                    self.add_reference(value, self.program.pc(), 2)?;
                    self.program.write(0x00);
                }
                _ => return Err(format!("only 16 and 8-bit labels are currently supported")),
            }
            Ok(())
        }
    }

    /// What prefix to use for a global name based on the local name
    fn scope_prefix(&self, level: usize) -> Result<String, String> {
        if level > self.scopes.len() {
            Err(String::from("Too many @ symbols"))
        } else {
            Ok(self.scopes[0..self.scopes.len() - level].join("__")
                + if self.scopes.len() - level > 0 {
                    "__"
                } else {
                    ""
                })
        }
    }

    fn add_tag(&mut self, name: &str, value: u16) -> Result<(), String> {
        let name = self.scope_prefix(0)? + name;
        self.program.program_mut().add_tag(name, value);
        Ok(())
    }

    /// Create a tag reference, resolving local names
    fn add_reference(&mut self, tag: &str, location: u16, size: u8) -> Result<(), String> {
        let mut real_name = String::new();
        let mut at_count = 0;
        for i in tag.chars() {
            if i == '@' {
                at_count += 1;
            } else {
                if at_count != 0 {
                    real_name += self.scope_prefix(at_count - 1)?.as_str();
                    at_count = 0;
                }
                real_name.push(i);
            }
        }
        self.program
            .program_mut()
            .add_reference(location, &real_name, size);
        Ok(())
    }
}
