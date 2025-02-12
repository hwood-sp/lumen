use liblumen_alloc::erts::exception::Exception;
use liblumen_alloc::Term;

use super::atom::bytes_len_try_into_term;
use super::u8;

pub fn decode<'a>(safe: bool, bytes: &'a [u8]) -> Result<(Term, &'a [u8]), Exception> {
    let (len_u8, after_len_bytes) = u8::decode(bytes)?;
    let len_usize = len_u8 as usize;

    bytes_len_try_into_term(safe, after_len_bytes, len_usize)
}
