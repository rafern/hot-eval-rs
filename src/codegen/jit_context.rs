use inkwell::context::Context;

use super::compilation_context::CompilationContext;

pub struct JITContext {
    llvm_context: Context,
    next: usize,
}

impl JITContext {
    pub fn new() -> Self {
        JITContext { llvm_context: Context::create(), next: 0 }
    }

    pub fn make_compilation_context(&'_ mut self) -> CompilationContext<'_> {
        let comp_ctx_id = self.next;
        self.next += 1;
        CompilationContext::new(&self.llvm_context, comp_ctx_id)
    }
}