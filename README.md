# hot-eval
JIT-compiled expression evaluation for hot paths

## What?
Essentially, it's a JIT-compiled toy language designed for single expressions.

Most expression evaluators convert a string to an AST, and then evaluate the
AST directly. Instead, this crate:
1. Converts a string to an AST by using [LALRPOP](https://crates.io/crates/lalrpop)
2. Does semantic analysis on the AST to support types and custom variables/functions, creating an analysed AST (AAST)
3. Converts the AAST to machine code by using [Inkwell](https://crates.io/crates/inkwell)

## Why?
For most use-cases, you **shouldn't** use this crate. Use a crate like
[evalexpr](https://crates.io/crates/evalexpr),
[fasteval](https://crates.io/crates/fasteval),
[meval](https://crates.io/crates/meval), etc... However, if you need to evaluate
the same expression repeatedly in a hot path, the expressions are trusted, and
you value performance over anything else (including ergonomics and safety), then
this crate might be suitable for your use-case.

For example, a good use-case would be to implement arbitrary predicates for huge
input spaces.

A bad use-case would be to use this for a calculator, since you only need to
evaluate the expression once (and the compilation cost is probably going to
outweight the standard AST evaluation cost) and performance isn't critical.

## Usage
For now, this crate isn't published, so you need to add it as a Git dependency.
This crate will be published when it becomes more ergonomic and safe.

Make sure to have LLVM 21 installed, as it's a requirement for Inkwell. There
are no plans to switch the backend compilation to Cranelift.

There is no documentation yet, and it will be written when the first version is
published. If you want to use this project anyway, see `main.rs` for example
code.

## Acknowledgements
Some of the terminology in this crate was inpired by other projects:
- The concept of a Table to define the global context was inspired by Lua
- The concept of a Slab to define a fast interface was inspired by fasteval