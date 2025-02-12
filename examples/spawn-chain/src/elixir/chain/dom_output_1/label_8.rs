use std::sync::Arc;

use liblumen_alloc::erts::exception::system::Alloc;
use liblumen_alloc::erts::process::code::stack::frame::{Frame, Placement};
use liblumen_alloc::erts::process::{code, Process};
use liblumen_alloc::erts::term::Term;

use crate::elixir::chain::dom_output_1::label_9;

pub fn place_frame_with_arguments(
    process: &Process,
    placement: Placement,
    document: Term,
    tr: Term,
) -> Result<(), Alloc> {
    process.stack_push(tr)?;
    process.stack_push(document)?;
    process.place_frame(frame(process), placement);

    Ok(())
}

// Private

/// ```elixir
/// # label 8
/// # pushed to stack: (document, tr)
/// # returned from call: text_text
/// # full stack: (text_text, document, tr)
/// # returns: {:ok, text_td}
/// {:ok, text_td} = Lumen::Web::Document.create_element(document, "td")
/// Lumen::Web::Node.append_child(text_td, text_text)
/// Lumen::Web::Node.append_child(tr, text_td)
///
/// {:ok, tbody} = Lumen::Web::Document.get_element_by_id(document, "output")
/// Lumen::Web::Node.append_child(tbody, tr)
/// ```
fn code(arc_process: &Arc<Process>) -> code::Result {
    arc_process.reduce();

    let text_text = arc_process.stack_pop().unwrap();
    assert!(text_text.is_resource_reference());
    let document = arc_process.stack_pop().unwrap();
    assert!(document.is_resource_reference());
    let tr = arc_process.stack_pop().unwrap();
    assert!(tr.is_resource_reference());

    label_9::place_frame_with_arguments(arc_process, Placement::Replace, document, tr, text_text)
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
