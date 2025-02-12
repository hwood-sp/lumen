// wasm32 proptest cannot be compiled at the same time as non-wasm32 proptest, so disable tests that
// use proptest completely for wasm32
//
// See https://github.com/rust-lang/cargo/issues/4866
#[cfg(all(not(target_arch = "wasm32"), test))]
mod test;

use std::convert::TryInto;
use std::u8;

use liblumen_alloc::badarg;
use liblumen_alloc::erts::exception;
use liblumen_alloc::erts::process::Process;
use liblumen_alloc::erts::term::binary::aligned_binary::AlignedBinary;
use liblumen_alloc::erts::term::binary::maybe_aligned_maybe_binary::MaybeAlignedMaybeBinary;
use liblumen_alloc::erts::term::{IterableBitstring, Term, TypedTerm};

use lumen_runtime_macros::native_implemented_function;

use crate::binary::ToTermOptions;
use crate::distribution::external_term_format::{term, VERSION_NUMBER};

macro_rules! maybe_aligned_maybe_binary_try_into_term {
    ($process:expr, $options:expr, $ident:expr) => {
        if $ident.is_binary() {
            if $ident.is_aligned() {
                versioned_tagged_bytes_try_into_term($process, $options, unsafe {
                    $ident.as_bytes()
                })
            } else {
                let byte_vec: Vec<u8> = $ident.full_byte_iter().collect();
                versioned_tagged_bytes_try_into_term($process, $options, &byte_vec)
            }
        } else {
            Err(badarg!().into())
        }
    };
}

#[native_implemented_function(binary_to_term/2)]
pub fn native(process: &Process, binary: Term, options: Term) -> exception::Result {
    let options: ToTermOptions = options.try_into()?;

    match binary.to_typed_term().unwrap() {
        TypedTerm::Boxed(boxed) => match boxed.to_typed_term().unwrap() {
            TypedTerm::HeapBinary(heap_binary) => {
                versioned_tagged_bytes_try_into_term(process, &options, heap_binary.as_bytes())
            }
            TypedTerm::MatchContext(match_context) => {
                maybe_aligned_maybe_binary_try_into_term!(process, &options, match_context)
            }
            TypedTerm::ProcBin(process_binary) => {
                versioned_tagged_bytes_try_into_term(process, &options, process_binary.as_bytes())
            }
            TypedTerm::SubBinary(subbinary) => {
                maybe_aligned_maybe_binary_try_into_term!(process, &options, subbinary)
            }
            _ => Err(badarg!().into()),
        },
        _ => Err(badarg!().into()),
    }
}

fn versioned_tagged_bytes_try_into_term(
    process: &Process,
    options: &ToTermOptions,
    bytes: &[u8],
) -> exception::Result {
    if 1 <= bytes.len() && bytes[0] == VERSION_NUMBER {
        let (term, after_term_bytes) = term::decode_tagged(process, options.existing, &bytes[1..])?;

        if options.used {
            let used_byte_len = bytes.len() - after_term_bytes.len();
            let used = process.integer(used_byte_len)?;

            process
                .tuple_from_slice(&[term, used])
                .map_err(|alloc| alloc.into())
        } else {
            Ok(term)
        }
    } else {
        Err(badarg!().into())
    }
}
