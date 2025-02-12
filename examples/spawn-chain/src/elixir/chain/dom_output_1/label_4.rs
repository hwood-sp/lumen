use std::sync::Arc;

use liblumen_alloc::erts::exception::system::Alloc;
use liblumen_alloc::erts::process::code::stack::frame::{Frame, Placement};
use liblumen_alloc::erts::process::{code, Process};
use liblumen_alloc::erts::term::Term;

use crate::elixir::chain::dom_output_1::label_5;

pub fn place_frame_with_arguments(
    process: &Process,
    placement: Placement,
    document: Term,
    tr: Term,
    text: Term,
) -> Result<(), Alloc> {
    assert!(text.is_binary());
    process.stack_push(text)?;
    process.stack_push(tr)?;
    process.stack_push(document)?;
    process.place_frame(frame(process), placement);

    Ok(())
}

// Private

/// ```elixir
/// # label 4
/// # pushed to stack: (document, tr, text)
/// # returned from call: pid_text
/// # full stack: (pid_text, document, tr, text)
/// # returns: {:ok, pid_td}
/// {:ok, pid_td} = Lumen::Web::Document.create_element(document, "td")
/// Lumen::Web::Node.append_child(pid_td, pid_text)
/// Lumen::Web::Node.append_child(tr, pid_td)
///
/// {:ok, text_text} = Lumen::Web::Document.create_text_node(document, to_string(text))
/// {:ok, text_td} = Lumen::Web::Document.create_element(document, "td")
/// Lumen::Web::Node.append_child(text_td, text_text)
/// Lumen::Web::Node.append_child(tr, text_td)
///
/// {:ok, tbody} = Lumen::Web::Document.get_element_by_id(document, "output")
/// Lumen::Web::Node.append_child(tbody, tr)
/// ```
fn code(arc_process: &Arc<Process>) -> code::Result {
    arc_process.reduce();

    let pid_text = arc_process.stack_pop().unwrap();
    assert!(pid_text.is_resource_reference());
    let document = arc_process.stack_pop().unwrap();
    assert!(document.is_resource_reference());
    let tr = arc_process.stack_pop().unwrap();
    assert!(tr.is_resource_reference());
    let text = arc_process.stack_pop().unwrap();

    label_5::place_frame_with_arguments(
        arc_process,
        Placement::Replace,
        document,
        tr,
        pid_text,
        text,
    )
    .unwrap();

    let tag = arc_process.binary_from_str("td").unwrap();
    lumen_web::document::create_element_2::place_frame_with_arguments(
        arc_process,
        Placement::Push,
        document,
        tag,
    )
    .unwrap();

    Process::call_code(arc_process)
}

fn frame(process: &Process) -> Frame {
    let module_function_arity = process.current_module_function_arity().unwrap();

    Frame::new(module_function_arity, code)
}
