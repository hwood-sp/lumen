//! ```elixir
//! case Lumen.Web.Window.window() do
//!    {:ok, window} -> ...
//!    :error -> ...
//! end
//! ```

use liblumen_alloc::erts::exception;
use liblumen_alloc::erts::process::Process;

use lumen_runtime_macros::native_implemented_function;

use crate::option_to_ok_tuple_or_error;

#[native_implemented_function(window/0)]
pub fn native(process: &Process) -> exception::Result {
    let option_window = web_sys::window();

    option_to_ok_tuple_or_error(process, option_window).map_err(|error| error.into())
}
