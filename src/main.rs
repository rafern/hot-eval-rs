use std::{error::Error, hint::black_box, time::Instant};

use hot_eval::{codegen::{compiled_expression::{CompiledExpression, HotEvalJitFunction}, jit_context::JITContext}, common::{binding::{Binding, BindingFunctionParameter}, slab::Slab, table::Table, value::Value, value_type::ValueType}};

const ITERS: u32 = 100_000_000;

#[inline(never)]
fn get_wanted_x(seed1: u32, seed2: u32, seed3: &u32) -> u32 {
    (seed1 * 123 - 45) / seed2 + seed3
}

#[inline(never)]
fn benchmark_jit<'ctx>(slab: &'ctx mut Slab, jit_fn: HotEvalJitFunction<'ctx, bool>, test_value_idx: usize, seed3_idx: usize) -> u32 {
    let mut matches = 0;

    for x in 0..ITERS {
        slab.set_value(test_value_idx, x);
        slab.set_ptr_value(seed3_idx, &42);
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

    let seed3_idx = table.add_hidden_state(ValueType::USize);

    table.add_binding("x".to_string(), Binding::Variable { value_type: ValueType::U32 })?;
    table.add_binding("get_wanted_x".to_string(), Binding::Function {
        ret_type: ValueType::U32,
        params: vec![
            BindingFunctionParameter::ConstArgument { value: Value::U32 { inner: 3 } },
            BindingFunctionParameter::Parameter { value_type: ValueType::U32 },
            BindingFunctionParameter::HiddenStateArgument { hidden_state_idx: seed3_idx, cast_to_type: None },
        ],
        fn_ptr: get_wanted_x as *const (),
    })?;

    if let CompiledExpression::Bool { mut slab, jit_fn } = comp_ctx.compile_str("x == get_wanted_x(2)", &table)? {
        let test_value_idx = slab.get_binding_index(&"x".to_string()).unwrap();
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