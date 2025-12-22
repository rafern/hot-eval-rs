use std::{error::Error, hint::black_box, time::Instant};

use hot_eval::{codegen::{compiled_expression::{CompiledExpression, HotEvalJitFunction}, jit_context::JITContext}, common::{binding::FnSpecCallArg, slab::Slab, table::Table, value_type::ValueType}};

const ITERS: u32 = 100_000_000;

#[inline(never)]
fn get_wanted_x(seed1: u32, seed2: u32, seed3: &u32) -> u32 {
    (seed1 * 123 - 45) / seed2 + *seed3
}

#[inline(never)]
fn benchmark_jit<'ctx>(slab: &'ctx mut Slab, jit_fn: HotEvalJitFunction<'ctx, bool>, test_value_idx: usize, seed3_idx: usize) -> u32 {
    let mut matches = 0;

    for x in 0..ITERS {
        slab.set_value(test_value_idx, x);
        unsafe { slab.set_ptr_value(seed3_idx, &42); }
        if unsafe { jit_fn.call() } {
            matches += 1;
        }
    }

    matches
}

#[inline(never)]
fn benchmark_aot_inline<'ctx>() -> u32 {
    let mut matches = 0;

    for x in 0..ITERS {
        // XXX black_box here to avoid optimizations on the predicate, to make
        //     the comparison fairer
        if black_box(x == black_box(get_wanted_x(black_box(3), black_box(2), black_box(&42)))) {
            matches += 1;
        }
    }

    matches
}

#[inline(never)]
fn benchmark_aot_closure<'ctx>(closure: impl Fn(u32) -> bool) -> u32 {
    let mut matches = 0;

    for x in 0..ITERS {
        // XXX black_box here to avoid optimizations on the predicate, to make
        //     the comparison fairer
        if black_box(closure(x)) {
            matches += 1;
        }
    }

    matches
}

#[inline(never)]
fn benchmark_aot_capturing_closure<'ctx>(closure: impl Fn(u32) -> bool) -> u32 {
    let mut matches = 0;

    for x in 0..ITERS {
        // XXX black_box here to avoid optimizations on the predicate, to make
        //     the comparison fairer
        if black_box(closure(x)) {
            matches += 1;
        }
    }

    matches
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut jit_ctx = JITContext::new();
    let mut comp_ctx = jit_ctx.make_compilation_context()?;
    let mut table = Table::new();

    let seed3_idx = table.add_ptr_hidden_state();

    table.add_variable("x".into(), ValueType::U32)?;
    table.add_function_3_map("get_wanted_x".into(), get_wanted_x, 3u32, FnSpecCallArg::MappedArgument { param_idx: 0 }, FnSpecCallArg::from_hidden_state(seed3_idx))?;

    if let CompiledExpression::Bool { mut slab, jit_fn } = comp_ctx.compile_str("x == get_wanted_x(2)", &table)? {
        let test_value_idx = slab.get_binding_index(&"x".into()).unwrap();
        let start = Instant::now();
        let matches = benchmark_jit(&mut slab, jit_fn, test_value_idx, seed3_idx);
        let secs = Instant::now().duration_since(start).as_secs_f64();
        println!("                  [jit] found {matches} matches in {secs} seconds");
    } else {
        panic!("expected a predicate, not any other type of expression");
    }

    {
        let start = Instant::now();
        let dummy = 1337u16;
        let matches = benchmark_aot_capturing_closure(|x| {
            black_box(dummy);
            x == black_box(get_wanted_x(black_box(3), black_box(2), black_box(&42)))
        });
        let secs = Instant::now().duration_since(start).as_secs_f64();
        println!("[aot_capturing_closure] found {matches} matches in {secs} seconds");
    }

    {
        let start = Instant::now();
        let matches = benchmark_aot_closure(|x| x == black_box(get_wanted_x(black_box(3), black_box(2), black_box(&42))));
        let secs = Instant::now().duration_since(start).as_secs_f64();
        println!("          [aot_closure] found {matches} matches in {secs} seconds");
    }

    {
        let start = Instant::now();
        let matches = benchmark_aot_inline();
        let secs = Instant::now().duration_since(start).as_secs_f64();
        println!("           [aot_inline] found {matches} matches in {secs} seconds");
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}