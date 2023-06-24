use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TagReference {
    pub tag: String,
    pub location: u16,
    /// size in bytes
    pub size: u8,
}

#[derive(Debug, Clone)]
pub struct Tag {
    pub value: u16,
}

#[derive(Debug)]
/// The program holds machine code and tags
pub struct Program {
    pub tags: HashMap<String, Tag>,
    pub references: Vec<TagReference>,
    pub code: Vec<u8>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            tags: HashMap::new(),
            code: Vec::new(),
            references: Vec::new(),
        }
    }

    pub fn from(code: Vec<u8>, tags: HashMap<String, Tag>, references: Vec<TagReference>) -> Self {
        Self {
            tags,
            references,
            code,
        }
    }

    /// Create a tag with a given name and a given value
    pub fn add_tag(&mut self, name: String, value: u16) {
        self.tags.insert(name, Tag { value });
    }

    /// Refer to a tag.
    /// The byte will be filled with the value of the tag when the program is linked.
    pub fn add_reference(&mut self, location: u16, tag: &str, size: u8) {
        self.references.push(TagReference {
            tag: String::from(tag),
            location,
            size,
        });
    }

    /// Output the code into a file
    pub fn dump(self, file: &mut dyn std::io::Write) -> Result<(), String> {
        if self.references.len() > 0 {
            let mut r = String::from("unresolved references:\n");
            for i in self.references {
                r += "\t";
                r += i.tag.as_str();
                r += "\n";
            }
            return Err(r);
        }
        file.write_all(&self.code[..])
            .expect("Error writing to output file");
        Ok(())
    }
}

pub struct ProgramWriter {
    program: Program,
    /// The current program counter (where code is inserted)
    pc: usize,
    /// The stack for remembering the location of the program counter (sometimes useful when seeking a lot)
    pc_stack: Vec<usize>,
}

impl ProgramWriter {
    pub fn new() -> Self {
        Self {
            program: Program::new(),
            pc: 0,
            pc_stack: Vec::new(),
        }
    }

    /// Add a byte of code at the current location of the program counter
    pub fn write(&mut self, byte: u8) {
        if self.pc as isize > self.program.code.len() as isize - 1 {
            self.program.code.resize(self.pc + 1, 0x00);
        }
        self.program.code[self.pc] = byte;
        self.pc += 1;
    }

    /// Get the value of the program counter
    pub fn pc(&self) -> u16 {
        self.pc as _
    }

    /// Set the program counter
    pub fn seek(&mut self, position: u16) {
        self.pc = position as _;
    }

    /// Save the program counter
    pub fn push_pc(&mut self) {
        self.pc_stack.push(self.pc);
    }

    /// Restore the program counter
    pub fn pop_pc(&mut self) {
        self.pc = self
            .pc_stack
            .pop()
            .expect("Tried to pop PC with empty stack!");
    }

    #[allow(unused)]
    pub fn program(&self) -> &Program {
        &self.program
    }

    pub fn program_mut(&mut self) -> &mut Program {
        &mut self.program
    }

    pub fn into_program(self) -> Program {
        self.program
    }
}
