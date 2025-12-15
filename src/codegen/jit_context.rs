use inkwell::{context::Context, execution_engine::ExecutionEngine, support::LLVMString};

use super::compilation_context::CompilationContext;

pub struct JITContext {
    llvm_context: Context,
    next: usize,
}

impl JITContext {
    pub fn new() -> Self {
        // HACK this is needed due to a bug in inkwell where compiling a project
        //      with LTO causes a segfault:
        //      https://github.com/TheDan64/inkwell/issues/320
        ExecutionEngine::link_in_mc_jit();
        JITContext { llvm_context: Context::create(), next: 0 }
    }

    pub fn make_compilation_context(&'_ mut self) -> Result<CompilationContext<'_>, LLVMString> {
        let comp_ctx_id = self.next;
        self.next += 1;
        CompilationContext::new(&self.llvm_context, comp_ctx_id)
    }
}