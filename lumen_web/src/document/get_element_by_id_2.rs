//! ```elixir
//! case Lumen.Web.Document.get_element_by_id(document, "element-id") do
//!   {:ok, element} -> ...
//!   :error -> ...
//! end
//! ```

use liblumen_alloc::erts::exception;
use liblumen_alloc::erts::process::Process;
use liblumen_alloc::erts::term::Term;

use lumen_runtime_macros::native_implemented_function;

use lumen_runtime::binary_to_string::binary_to_string;

use crate::document::document_from_term;
use crate::option_to_ok_tuple_or_error;

#[native_implemented_function(get_element_by_id/2)]
pub fn native(process: &Process, document: Term, id: Term) -> exception::Result {
    let document_document = document_from_term(document)?;
    let id_string: String = binary_to_string(id)?;

    option_to_ok_tuple_or_error(process, document_document.get_element_by_id(&id_string))
        .map_err(|error| error.into())
}
