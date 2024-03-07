use crate::{jit::asm, parser::Operation};
use anyhow::Result;

const REG_MEMORY: asm::AsmRegister64 = asm::rsi;

pub fn emit(ops: Vec<Operation>, code: &mut asm::CodeAssembler) -> Result<()> {
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    #[cfg(not(target_arch = "x86_64"))]
    #[cfg(not(target_pointer_width = "64"))]
    panic!("JIT is only supported on x86_64 Linux and macOS.");

    code.mov(REG_MEMORY, asm::rdi)?;

    #[cfg(not(target_os = "macos"))]
    code.mov(asm::rdx, 1_u64)?; // OPTIMIZATION: rdx is only used for the length in read/write syscalls. it can always be 1

    emit_operations(ops, code)?;
    code.ret()?;

    Ok(())
}

fn emit_operations(ops: Vec<Operation>, code: &mut asm::CodeAssembler) -> Result<()> {
    for op in ops.iter() {
        match op {
            Operation::Left(n) => {
                code.sub(REG_MEMORY, (n * 4) as i32)?;
            }
            Operation::Right(n) => {
                code.add(REG_MEMORY, (n * 4) as i32)?;
            }
            Operation::Increment(n) => {
                code.add(asm::dword_ptr(REG_MEMORY), *n)?;
            }
            Operation::Decrement(n) => {
                code.sub(asm::dword_ptr(REG_MEMORY), *n)?;
            }
            Operation::Output => {
                #[cfg(target_os = "macos")]
                code.mov(asm::rax, 0x2000004_u64)?; // syscall number
                #[cfg(target_os = "linux")]
                code.mov(asm::rax, 0x1_u64)?; // syscall number
                code.mov(asm::rdi, 1_u64)?; // file descriptor

                // OPTIMIZATION: if REG_MEMORY is already rsi, we don't need to move it
                if REG_MEMORY != asm::rsi {
                    code.mov(asm::rsi, REG_MEMORY)?;
                }

                #[cfg(target_os = "macos")]
                code.mov(asm::rdx, 1_u64)?; // length; must be reset only on macos

                code.syscall()?;
            }
            Operation::Input => {
                #[cfg(target_os = "macos")]
                code.mov(asm::rax, 0x2000003_u64)?; // syscall number
                #[cfg(target_os = "linux")]
                code.mov(asm::rax, 0x0_u64)?; // syscall number
                code.mov(asm::rdi, 0_u64)?; // file descriptor

                if REG_MEMORY != asm::rsi {
                    code.mov(asm::rsi, REG_MEMORY)?;
                }

                #[cfg(target_os = "macos")]
                code.mov(asm::rdx, 1_u64)?; // length; must be reset only on macos

                code.syscall()?;
            }
            Operation::Loop(inner_ops) => {
                let mut start = code.create_label();
                let mut end = code.create_label();

                code.cmp(asm::dword_ptr(REG_MEMORY), 0)?;
                code.je(end)?;

                code.set_label(&mut start)?;

                emit_operations(inner_ops.clone(), code)?;

                code.cmp(asm::dword_ptr(REG_MEMORY), 0)?;
                code.jne(start)?;

                code.set_label(&mut end)?;
            }
        }
    }

    Ok(())
}
