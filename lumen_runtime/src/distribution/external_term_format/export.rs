use liblumen_alloc::erts::exception::Exception;
use liblumen_alloc::{Process, Term};

use crate::code;

use super::{atom, small_integer};

pub fn decode<'a>(
    process: &Process,
    safe: bool,
    bytes: &'a [u8],
) -> Result<(Term, &'a [u8]), Exception> {
    let (module, after_module_bytes) = atom::decode_tagged(safe, bytes)?;
    let (function, after_function_bytes) = atom::decode_tagged(safe, after_module_bytes)?;
    let (arity, after_arity_bytes) = small_integer::decode_tagged_u8(after_function_bytes)?;

    let option_code = code::export::get(&module, &function, arity);

    let closure = process.export_closure(module, function, arity, option_code)?;

    Ok((closure, after_arity_bytes))
}
