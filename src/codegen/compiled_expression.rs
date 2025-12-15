use inkwell::execution_engine::JitFunction;

use crate::common::slab::Slab;

pub type HotEvalJitFunction<'ctx, T> = JitFunction<'ctx, unsafe extern "C" fn() -> T>;

pub enum CompiledExpression<'ctx> {
    U8 { slab: Slab, jit_fn: HotEvalJitFunction<'ctx, u8> },
    U16 { slab: Slab, jit_fn: HotEvalJitFunction<'ctx, u16> },
    U32 { slab: Slab, jit_fn: HotEvalJitFunction<'ctx, u32> },
    U64 { slab: Slab, jit_fn: HotEvalJitFunction<'ctx, u64> },
    USize { slab: Slab, jit_fn: HotEvalJitFunction<'ctx, usize> },
    I8 { slab: Slab, jit_fn: HotEvalJitFunction<'ctx, i8> },
    I16 { slab: Slab, jit_fn: HotEvalJitFunction<'ctx, i16> },
    I32 { slab: Slab, jit_fn: HotEvalJitFunction<'ctx, i32> },
    I64 { slab: Slab, jit_fn: HotEvalJitFunction<'ctx, i64> },
    F32 { slab: Slab, jit_fn: HotEvalJitFunction<'ctx, f32> },
    F64 { slab: Slab, jit_fn: HotEvalJitFunction<'ctx, f64> },
    Bool { slab: Slab, jit_fn: HotEvalJitFunction<'ctx, bool> },
}