use std::sync::Arc;

use liblumen_alloc::erts::exception::system::Alloc;
use liblumen_alloc::erts::exception::Exception;
use liblumen_alloc::erts::process::code;
use liblumen_alloc::erts::process::code::stack::frame::{Frame, Placement};
use liblumen_alloc::erts::process::Process;
use liblumen_alloc::erts::term::Term;
use liblumen_alloc::erts::term::{atom_unchecked, Atom};
use liblumen_alloc::erts::ModuleFunctionArity;

use lumen_runtime::binary_to_string::binary_to_string;
use lumen_runtime::system;

pub fn place_frame_with_arguments(
    process: &Process,
    placement: Placement,
    binary: Term,
) -> Result<(), Alloc> {
    assert!(binary.is_binary());

    process.stack_push(binary)?;
    process.place_frame(frame(), placement);

    Ok(())
}

// Private

fn code(arc_process: &Arc<Process>) -> code::Result {
    arc_process.reduce();

    let elixir_string = arc_process.stack_pop().unwrap();

    match binary_to_string(elixir_string) {
        Ok(string) => {
            // NOT A DEBUGGING LOG
            system::io::puts(&string);

            let ok = atom_unchecked("ok");
            arc_process.return_from_call(ok).unwrap();

            Process::call_code(arc_process)
        }
        Err(exception) => match exception {
            Exception::Runtime(runtime_exception) => {
                arc_process.exception(runtime_exception);

                Ok(())
            }
            Exception::System(system_exception) => Err(system_exception),
        },
    }
}

fn frame() -> Frame {
    let module_function_arity = Arc::new(ModuleFunctionArity {
        module: super::module(),
        function: function(),
        arity: 1,
    });

    Frame::new(module_function_arity, code)
}

fn function() -> Atom {
    Atom::try_from_str("puts").unwrap()
}
