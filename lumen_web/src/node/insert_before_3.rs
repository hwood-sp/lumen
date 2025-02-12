//! Unlike [Node.appendChild](https://developer.mozilla.org/en-US/docs/Web/API/Node/appendChild),
//! this does not return the appended child as that is prone to errors with chaining.
//!
//! ```elixir
//! case Lumen.Web.Node.insert_before(parent, new_child, reference_child) do
//!   {:ok, inserted_child} -> ...
//!   {:error, :hierarchy_request} -> ...
//!   {:error, :wrong_document} -> ...
//!   {:error, :no_modification_allowed} -> ...
//!   {:error, :not_found} -> ...
//!   {:error, :not_supported} -> ...
//! end
//! ```

use wasm_bindgen::JsCast;

use web_sys::DomException;

use liblumen_alloc::erts::exception;
use liblumen_alloc::erts::process::Process;
use liblumen_alloc::erts::term::{atom_unchecked, Term};

use lumen_runtime_macros::native_implemented_function;

use crate::node::node_from_term;
use crate::{error, ok_tuple};

#[native_implemented_function(insert_before/3)]
fn native(
    process: &Process,
    parent: Term,
    new_child: Term,
    reference_child: Term,
) -> exception::Result {
    let parent_node = node_from_term(parent)?;
    let new_child_node = node_from_term(new_child)?;

    let option_reference_child_node = if reference_child == atom_unchecked("nil") {
        None
    } else {
        Some(node_from_term(reference_child)?)
    };

    match parent_node.insert_before(new_child_node, option_reference_child_node) {
        Ok(inserted_child_node) => {
            ok_tuple(process, Box::new(inserted_child_node)).map_err(|error| error.into())
        }
        Err(js_value) => {
            let dom_exception = js_value.dyn_into::<DomException>().unwrap();

            let reason_name = match dom_exception.name().as_str() {
                "HierarchyRequestError" => "hierarchy_request",
                "NoModificationAllowedError" => "no_modification_allowed",
                "NotFoundError" => "not_found",
                "NotSupportedError" => "not_supported",
                "WrongDocumentError" => "wrong_document",
                name => unimplemented!(
                    "Convert DOMException with name ({:?}) to error reason",
                    name
                ),
            };
            let reason = atom_unchecked(reason_name);
            let error_tuple = process.tuple_from_slice(&[error(), reason])?;

            Ok(error_tuple)
        }
    }
}
