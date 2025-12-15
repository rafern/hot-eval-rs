use std::error::Error;

use inkwell::{OptimizationLevel, context::Context, execution_engine::ExecutionEngine, module::Module, support::LLVMString};

use crate::{analysis::packed_analysis_tree::PackedAnalysisTree, ast::ast_node::Expression, codegen::{codegen_context::CodegenContext, ir_value::IRValue, ir_value_type::IRValueType}, common::{slab::Slab, table::Table, value_type::ValueType}};

use super::compiled_expression::CompiledExpression;

pub struct CompilationContext<'ctx> {
    execution_engine: ExecutionEngine<'ctx>,
    module: Module<'ctx>,
    llvm_context: &'ctx Context,
    next: usize,
}

impl<'ctx> CompilationContext<'ctx> {
    pub fn new(llvm_context: &'ctx Context, comp_ctx_id: usize) -> Result<Self, LLVMString> {
        let module = llvm_context.create_module(&format!("hot_eval_module_{comp_ctx_id}"));
        let execution_engine = module.create_jit_execution_engine(OptimizationLevel::Aggressive)?;
        Ok(Self { llvm_context, module, execution_engine, next: 0 })
    }

    pub fn compile_analysed_ast(&mut self, aast: PackedAnalysisTree, slab: Slab) -> Result<CompiledExpression<'_>, Box<dyn Error>> {
        let id = self.next;
        self.next += 1;

        let builder = self.llvm_context.create_builder();

        let fn_name = format!("hot_eval_fn_{id}");
        let fn_ast_type = aast.get_expr_type()?;
        let fn_type = match IRValueType::from_value_type(&fn_ast_type, &self.llvm_context) {
            IRValueType::Int { llvm, .. } => llvm.fn_type(&[], false),
            IRValueType::Float { llvm } => llvm.fn_type(&[], false),
        };
        let function = self.module.add_function(&fn_name, fn_type, None);
        let basic_block = self.llvm_context.append_basic_block(function, "entry");

        builder.position_at_end(basic_block);

        let codegen_ctx = CodegenContext {
            llvm_context: &self.llvm_context,
            module: &self.module,
            builder: &builder,
            execution_engine: &self.execution_engine,
            func: &function,
            slab: &slab,
        };
        let expr = IRValue::from_aast(&aast, &codegen_ctx)?;

        builder.build_return(Some(expr.ref_inner_generic()))?;

        // module.print_to_stderr();

        Ok(match fn_ast_type {
            // FIXME surely there's a better way than this, right?
            ValueType::U8 => CompiledExpression::U8 { slab, jit_fn: unsafe { self.execution_engine.get_function(&fn_name) }? },
            ValueType::U16 => CompiledExpression::U16 { slab, jit_fn: unsafe { self.execution_engine.get_function(&fn_name) }? },
            ValueType::U32 => CompiledExpression::U32 { slab, jit_fn: unsafe { self.execution_engine.get_function(&fn_name) }? },
            ValueType::U64 => CompiledExpression::U64 { slab, jit_fn: unsafe { self.execution_engine.get_function(&fn_name) }? },
            ValueType::USize => CompiledExpression::USize { slab, jit_fn: unsafe { self.execution_engine.get_function(&fn_name) }? },
            ValueType::I8 => CompiledExpression::I8 { slab, jit_fn: unsafe { self.execution_engine.get_function(&fn_name) }? },
            ValueType::I16 => CompiledExpression::I16 { slab, jit_fn: unsafe { self.execution_engine.get_function(&fn_name) }? },
            ValueType::I32 => CompiledExpression::I32 { slab, jit_fn: unsafe { self.execution_engine.get_function(&fn_name) }? },
            ValueType::I64 => CompiledExpression::I64 { slab, jit_fn: unsafe { self.execution_engine.get_function(&fn_name) }? },
            ValueType::F32 => CompiledExpression::F32 { slab, jit_fn: unsafe { self.execution_engine.get_function(&fn_name) }? },
            ValueType::F64 => CompiledExpression::F64 { slab, jit_fn: unsafe { self.execution_engine.get_function(&fn_name) }? },
            ValueType::Bool => CompiledExpression::Bool { slab, jit_fn: unsafe { self.execution_engine.get_function(&fn_name) }? },
        })
    }

    pub fn compile_ast(&mut self, ast: &Expression, table: &Table) -> Result<CompiledExpression<'_>, Box<dyn Error>> {
        let slab = Slab::from_table(table)?;
        let aast = PackedAnalysisTree::from_ast(ast, table)?;
        // aast.print_to_stderr();
        self.compile_analysed_ast(aast, slab)
    }

    pub fn compile_str<'src>(&mut self, source: &'src str, table: &Table) -> Result<CompiledExpression<'_>, Box<dyn Error + 'src>> {
        let ast = &Expression::from_src(source)?;
        // eprintln!("{:?}", ast);
        self.compile_ast(ast, table)
    }
}