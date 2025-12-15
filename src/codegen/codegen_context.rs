use inkwell::{builder::Builder, context::Context, execution_engine::ExecutionEngine, module::Module, values::FunctionValue};

use crate::common::slab::Slab;

pub struct CodegenContext<'ctx, 'build> {
    pub llvm_context: &'ctx Context,
    pub module: &'build Module<'ctx>,
    pub builder: &'build Builder<'ctx>,
    pub execution_engine: &'build ExecutionEngine<'ctx>,
    pub func: &'build FunctionValue<'ctx>,
    pub slab: &'build Slab,
}