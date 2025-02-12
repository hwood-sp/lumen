use std::sync::Arc;

use liblumen_alloc::erts::exception::system::Alloc;
use liblumen_alloc::erts::process::code::stack::frame::{Frame, Placement};
use liblumen_alloc::erts::process::{code, Process};
use liblumen_alloc::erts::term::Term;

use lumen_runtime::otp::erlang;

use crate::elixir::chain::counter_2::label_3;

pub fn place_frame_with_arguments(
    process: &Process,
    placement: Placement,
    next_pid: Term,
    output: Term,
) -> Result<(), Alloc> {
    process.stack_push(output)?;
    process.stack_push(next_pid)?;
    process.place_frame(frame(process), placement);

    Ok(())
}

// Private

/// ```elixir
/// # label 2
/// # pushed to stack: (next_pid, output)
/// # returned from call: sum
/// # full stack: (sum, next_pid, output)
/// # returns: sent
/// sent = send(next_pid, sum)
/// output.("sent #{sent} to #{next_pid}")
/// ```
fn code(arc_process: &Arc<Process>) -> code::Result {
    arc_process.reduce();

    let sum = arc_process.stack_pop().unwrap();
    assert!(sum.is_integer());
    let next_pid = arc_process.stack_pop().unwrap();
    assert!(next_pid.is_pid());
    let output = arc_process.stack_pop().unwrap();
    assert!(output.is_function());

    // ```elixir
    // # label 3
    // # pushed stack: (output, next_pid)
    // # returned from call: sent
    // # full stack: (sent, output, next_pid)
    // # returns: :ok
    // sent = ...
    // output.("sent #{sent} to #{next_pid}")
    // ```
    label_3::place_frame_with_arguments(arc_process, Placement::Replace, output, next_pid).unwrap();

    // ```elixir
    // # pushed stack: (next_pid, sum)
    // # returned from call: N/A
    // # full stack: (next_pid, sum)
    // # returns: sent
    // send(next_pid, sum)
    erlang::send_2::place_frame_with_arguments(arc_process, Placement::Push, next_pid, sum)
        .unwrap();

    Process::call_code(arc_process)
}

fn frame(process: &Process) -> Frame {
    let module_function_arity = process.current_module_function_arity().unwrap();

    Frame::new(module_function_arity, code)
}
