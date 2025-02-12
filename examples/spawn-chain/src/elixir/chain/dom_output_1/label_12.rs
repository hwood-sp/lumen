use std::convert::TryInto;
use std::sync::Arc;

use liblumen_alloc::erts::exception::system::Alloc;
use liblumen_alloc::erts::process::code::stack::frame::{Frame, Placement};
use liblumen_alloc::erts::process::{code, Process};
use liblumen_alloc::erts::term::{atom_unchecked, Boxed, Term, Tuple};

pub fn place_frame_with_arguments(
    process: &Process,
    placement: Placement,
    tr: Term,
) -> Result<(), Alloc> {
    process.stack_push(tr)?;
    process.place_frame(frame(process), placement);

    Ok(())
}

// Private

/// ```elixir
/// # label 12
/// # pushed to stack: (tr)
/// # returned from call: {:ok, tbody}
/// # full stack: ({:ok, tbody}, tr)
/// # returns: :ok
/// Lumen::Web::Node.append_child(tbody, tr)
/// ```
fn code(arc_process: &Arc<Process>) -> code::Result {
    arc_process.reduce();

    let ok_tbody = arc_process.stack_pop().unwrap();
    assert!(
        ok_tbody.is_tuple(),
        "ok_tbody ({:?}) is not a tuple",
        ok_tbody
    );
    let tr = arc_process.stack_pop().unwrap();
    assert!(tr.is_resource_reference());

    let ok_tbody_tuple: Boxed<Tuple> = ok_tbody.try_into().unwrap();
    assert_eq!(ok_tbody_tuple.len(), 2);
    assert_eq!(ok_tbody_tuple[0], atom_unchecked("ok"));
    let tbody = ok_tbody_tuple[1];
    assert!(tbody.is_resource_reference());

    lumen_web::node::append_child_2::place_frame_with_arguments(
        arc_process,
        Placement::Replace,
        tbody,
        tr,
    )
    .unwrap();

    Process::call_code(arc_process)
}

fn frame(process: &Process) -> Frame {
    let module_function_arity = process.current_module_function_arity().unwrap();

    Frame::new(module_function_arity, code)
}
