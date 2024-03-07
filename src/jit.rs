use anyhow::Result;

pub use iced_x86::code_asm as asm;
use iced_x86::code_asm::CodeAssembler;

#[cfg(debug_assertions)]
use iced_x86::{Formatter, IntelFormatter};

const PAGE_SIZE: usize = 4096;

pub struct JitProgram {
    page: Option<*mut u8>,
    func: Option<fn(*mut i32)>,
    assembler: CodeAssembler,
}

impl JitProgram {
    pub fn new() -> Result<Self> {
        let assembler = CodeAssembler::new(64)?;

        Ok(JitProgram {
            func: None,
            page: None,
            assembler,
        })
    }

    pub fn code(&mut self) -> &mut CodeAssembler {
        &mut self.assembler
    }

    fn build(&mut self) -> Result<fn(*mut i32)> {
        if let Some(func) = self.func {
            return Ok(func);
        }

        let empty_page = unsafe {
            #[allow(invalid_value)]
            let page: *mut libc::c_void = std::mem::MaybeUninit::uninit().assume_init();
            page
        };

        let bytes = self.assembler.assemble(empty_page as u64)?;

        let page_count = (bytes.len() as f64 / PAGE_SIZE as f64).ceil() as usize;
        let size = page_count * PAGE_SIZE;

        let page: *mut u8 = unsafe {
            #[allow(invalid_value)]
            let mut page: *mut libc::c_void = std::mem::MaybeUninit::uninit().assume_init();
            libc::posix_memalign(&mut page, PAGE_SIZE, size);
            libc::mprotect(
                page,
                size,
                libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE,
            );

            libc::memset(page, 0xc3, size); // fil with ret

            page as *mut u8
        };

        let bytes = self.assembler.assemble(page as u64)?;

        for (i, byte) in bytes.iter().enumerate() {
            unsafe {
                page.offset(i as isize).write(*byte);
            };
        }

        let func = unsafe { std::mem::transmute(page) };
        self.page = Some(page);
        self.func = Some(func);

        Ok(func)
    }

    pub fn run(&mut self, memory: &mut [i32]) -> Result<()> {
        let run_program = self.build()?;
        run_program(memory.as_mut_ptr());
        Ok(())
    }

    #[cfg(debug_assertions)]
    pub fn format_asm(&self) -> String {
        let mut formatter = IntelFormatter::new();
        let mut output = String::new();

        for instruction in self.assembler.instructions() {
            formatter.format(instruction, &mut output);
            output.push('\n');
        }

        output
    }
}

impl Drop for JitProgram {
    fn drop(&mut self) {
        if let Some(page) = self.page {
            unsafe {
                libc::free(page as *mut libc::c_void);
            }
        }
    }
}
