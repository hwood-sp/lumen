use std::convert::TryInto;
use std::sync::Arc;

use liblumen_alloc::erts::process::code::stack::frame::{Frame, Placement};
use liblumen_alloc::erts::process::{code, Process};
use liblumen_alloc::erts::term::atom_unchecked;

use lumen_runtime::otp::erlang;

/// ```elixir
/// # label: 4
/// # pushed to stack: ()
/// # returned from call: n
/// # full stack: (n)
/// # returns: {time, value}
/// :erlang.spawn_opt(Chain, dom, [n], [min_heap_size: 79 + n * 10])
/// ```
pub fn place_frame(process: &Process, placement: Placement) {
    process.place_frame(frame(process), placement);
}

// Private

fn code(arc_process: &Arc<Process>) -> code::Result {
    arc_process.reduce();

    let n = arc_process.stack_pop().unwrap();
    assert!(n.is_integer());
    let n_usize: usize = n.try_into().unwrap();

    erlang::spawn_opt_4::place_frame_with_arguments(
        arc_process,
        Placement::Replace,
        atom_unchecked("Elixir.Chain"),
        atom_unchecked("dom"),
        arc_process.list_from_slice(&[n]).unwrap(),
        arc_process
            .list_from_slice(&[arc_process
                .tuple_from_slice(&[
                    atom_unchecked("min_heap_size"),
                    arc_process.integer(79 + n_usize * 10).unwrap(),
                ])
                .unwrap()])
            .unwrap(),
    )
    .unwrap();

    Process::call_code(arc_process)
}

fn frame(process: &Process) -> Frame {
    let module_function_arity = process.current_module_function_arity().unwrap();

    Frame::new(module_function_arity, code)
}
