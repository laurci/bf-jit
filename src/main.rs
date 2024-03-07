mod emitter;
mod jit;
mod parser;

use anyhow::{bail, Result};

use emitter::emit;
use jit::JitProgram;
use parser::parse_and_optimize_input;

const MEMORY_SIZE: usize = 5_000;

fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();

    if args.len() != 2 {
        bail!("Usage: {} <file>", args[0]);
    }

    let input = std::fs::read_to_string(&args[1])?;
    let ops = parse_and_optimize_input(&input);

    #[cfg(debug_assertions)]
    println!("Ops: {:?}\n", ops);

    let mut program = JitProgram::new()?;
    let code = program.code();

    emit(ops, code)?;

    #[cfg(debug_assertions)]
    println!("ASM: \n{}", program.format_asm());

    let mut memory = vec![0i32; MEMORY_SIZE];

    #[cfg(debug_assertions)]
    println!("Output:");
    program.run(&mut memory)?;

    #[cfg(debug_assertions)]
    println!("\nmemory[..100]: {:?}", &memory[..100]);

    Ok(())
}
