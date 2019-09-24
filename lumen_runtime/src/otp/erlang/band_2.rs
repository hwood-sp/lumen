// wasm32 proptest cannot be compiled at the same time as non-wasm32 proptest, so disable tests that
// use proptest completely for wasm32
//
// See https://github.com/rust-lang/cargo/issues/4866
#[cfg(all(not(target_arch = "wasm32"), test))]
mod test;

use liblumen_alloc::erts::exception;
use liblumen_alloc::erts::process::Process;
use liblumen_alloc::erts::term::Term;

use lumen_runtime_macros::native_implemented_function;

/// `band/2` infix operator.
#[native_implemented_function(band/2)]
pub fn native(process: &Process, left_integer: Term, right_integer: Term) -> exception::Result {
    bitwise_infix_operator!(left_integer, right_integer, process, &)
}
