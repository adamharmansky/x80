mod assembler;
mod linker;
mod parser;
mod program;
mod util;

use assembler::Assembler;
use linker::Linker;
use parser::Parser;
use program::Program;

use util::OrDie;

fn main() {
    let mut args = std::env::args();
    let name = args
        .next()
        .expect("Does this executable not fking have a name?");
    let output_file = args
        .next()
        .or_die(&format!("USAGE: {name} OUTPUT INPUT1 [INPUT2 ..]"));

    let mut linker = Linker::new();
    let mut asm = Assembler::new();
    for i in args {
        let file = Box::new(std::fs::File::open(i).or_die("Cannot open input file!"));
        let code = Parser::new(file);

        for i in code {
            match asm.parse(i.1) {
                Ok(()) => (),
                Err(e) => {
                    eprintln!("\x1b[91mline {}: {}\x1b[0m", i.0 + 1, e);
                    std::process::exit(1);
                }
            }
        }
    }

    linker.set_program(asm.program());
    let program = match linker.run() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("\x1b[91mLinker error: {}\x1b[0m", e);
            std::process::exit(1);
        }
    };

    let mut file = std::fs::File::create(output_file).or_die("Unable to create output file!");
    match program.dump(&mut file) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("\x1b[91mUnable to export program: {}\x1b[0m", e);
            std::process::exit(1);
        }
    }
}
