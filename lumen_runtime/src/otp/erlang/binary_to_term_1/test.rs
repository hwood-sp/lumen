use proptest::prop_assert_eq;
use proptest::test_runner::{Config, TestRunner};

use liblumen_alloc::badarg;
use liblumen_alloc::erts::process::Process;
use liblumen_alloc::erts::term::{atom_unchecked, Term};

use crate::otp::erlang::binary_to_term_1::native;
use crate::scheduler::with_process_arc;
use crate::test::strategy;

#[test]
fn without_binary_errors_badarg() {
    with_process_arc(|arc_process| {
        TestRunner::new(Config::with_source_file(file!()))
            .run(
                &strategy::term::is_not_binary(arc_process.clone()),
                |binary| {
                    prop_assert_eq!(native(&arc_process, binary), Err(badarg!().into()));

                    Ok(())
                },
            )
            .unwrap();
    });
}

#[test]
fn with_binary_encoding_atom_returns_atom() {
    with_binary_returns_term(
        // :erlang.term_to_binary(:atom)
        vec![131, 100, 0, 4, 97, 116, 111, 109],
        |_| atom_unchecked("atom"),
    );
}

#[test]
fn with_binary_encoding_empty_list_returns_empty_list() {
    with_binary_returns_term(
        // :erlang.term_to_binary([])
        vec![131, 106],
        |_| Term::NIL,
    );
}

#[test]
fn with_binary_encoding_list_returns_list() {
    with_binary_returns_term(
        // :erlang.term_to_binary([:zero, 1])
        vec![
            131, 108, 0, 0, 0, 2, 100, 0, 4, 122, 101, 114, 111, 97, 1, 106,
        ],
        |process| {
            process
                .cons(
                    atom_unchecked("zero"),
                    process
                        .cons(process.integer(1).unwrap(), Term::NIL)
                        .unwrap(),
                )
                .unwrap()
        },
    );
}

#[test]
fn with_binary_encoding_small_integer_returns_small_integer() {
    with_binary_returns_term(
        // :erlang.term_to_binary(0)
        vec![131, 97, 0],
        |process| process.integer(0).unwrap(),
    );
}

#[test]
fn with_binary_encoding_integer_returns_integer() {
    with_binary_returns_term(
        // :erlang.term_to_binary(-2147483648)
        vec![131, 98, 128, 0, 0, 0],
        |process| process.integer(-2147483648_isize).unwrap(),
    );
}

#[test]
fn with_binary_encoding_new_float_returns_float() {
    with_binary_returns_term(
        // :erlang.term_to_binary(1.0)
        vec![131, 70, 63, 240, 0, 0, 0, 0, 0, 0],
        |process| process.float(1.0).unwrap(),
    );
}

#[test]
fn with_binary_encoding_small_tuple_returns_tuple() {
    with_binary_returns_term(
        // :erlang.term_to_binary({:zero, 1})
        vec![131, 104, 2, 100, 0, 4, 122, 101, 114, 111, 97, 1],
        |process| {
            process
                .tuple_from_slice(&[atom_unchecked("zero"), process.integer(1).unwrap()])
                .unwrap()
        },
    );
}

#[test]
fn with_binary_encoding_byte_list_returns_list() {
    with_binary_returns_term(
        // :erlang.term_to_binary([?0, ?1])
        vec![131, 107, 0, 2, 48, 49],
        |process| {
            process
                .cons(
                    process.integer(48).unwrap(),
                    process
                        .cons(process.integer(49).unwrap(), Term::NIL)
                        .unwrap(),
                )
                .unwrap()
        },
    );
}

#[test]
fn with_binary_encoding_binary_returns_binary() {
    with_binary_returns_term(
        // :erlang.term_to_binary(<<0, 1>>)
        vec![131, 109, 0, 0, 0, 2, 0, 1],
        |process| process.binary_from_bytes(&[0, 1]).unwrap(),
    );
}

#[test]
fn with_binary_encoding_small_big_integer_returns_big_integer() {
    with_binary_returns_term(
        // :erlang.term_to_binary(4294967295)
        vec![131, 110, 4, 0, 255, 255, 255, 255],
        |process| process.integer(4294967295_usize).unwrap(),
    );
}

#[test]
fn with_binary_encoding_bit_string_returns_subbinary() {
    with_binary_returns_term(
        // :erlang.term_to_binary(<<1, 2::3>>)
        vec![131, 77, 0, 0, 0, 2, 3, 1, 64],
        |process| {
            process
                .subbinary_from_original(
                    process.binary_from_bytes(&[1, 0b010_00000]).unwrap(),
                    0,
                    0,
                    1,
                    3,
                )
                .unwrap()
        },
    );
}

#[test]
fn with_binary_encoding_small_atom_utf8_returns_atom() {
    with_binary_returns_term(
        // :erlang.term_to_binary(:"😈")
        vec![131, 119, 4, 240, 159, 152, 136],
        |_| atom_unchecked("😈"),
    );
}

fn with_binary_returns_term<T>(byte_vec: Vec<u8>, term: T)
where
    T: Fn(&Process) -> Term,
{
    with_process_arc(|arc_process| {
        TestRunner::new(Config::with_source_file(file!()))
            .run(
                &strategy::term::binary::containing_bytes(byte_vec, arc_process.clone()),
                |binary| {
                    prop_assert_eq!(native(&arc_process, binary), Ok(term(&arc_process)));

                    Ok(())
                },
            )
            .unwrap();
    });
}
