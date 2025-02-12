//! ```elixir
//! case Lumen.Web.Element.set_attribute(element, "data-attribute", "data-value") do
//!   :ok -> ...
//!   {:error, {:name, name} -> ...
//! end
//! ```

use liblumen_alloc::erts::exception;
use liblumen_alloc::erts::term::Term;

use lumen_runtime_macros::native_implemented_function;

use crate::{element, ok};

#[native_implemented_function(remove/1)]
fn native(element_term: Term) -> exception::Result {
    let element = element::from_term(element_term)?;

    element.remove();

    Ok(ok())
}
