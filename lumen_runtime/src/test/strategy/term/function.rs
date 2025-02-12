pub mod anonymous;
pub mod export;

use std::sync::Arc;

use num_bigint::BigInt;

use proptest::prop_oneof;
use proptest::strategy::{BoxedStrategy, Just, Strategy};

use liblumen_alloc::erts::process::code::{self, Code, DebuggableCode};
use liblumen_alloc::erts::term::{Atom, Term, TypedTerm};
use liblumen_alloc::erts::Process;

use crate::test::strategy;

pub fn module() -> BoxedStrategy<Term> {
    super::atom()
}

pub fn module_atom() -> BoxedStrategy<Atom> {
    strategy::atom()
}

pub fn function() -> BoxedStrategy<Term> {
    super::atom()
}

pub fn anonymous(arc_process: Arc<Process>) -> BoxedStrategy<Term> {
    (
        module_atom(),
        anonymous::index(),
        anonymous::old_unique(),
        anonymous::unique(),
        arity_u8(),
        anonymous::creator(),
        option_debuggable_code(),
    )
        .prop_map(
            move |(module, index, old_unique, unique, arity, creator, option_debuggable_code)| {
                let option_code: Option<Code> =
                    option_debuggable_code.map(|debuggable_code| debuggable_code.0);

                if let Some(code) = option_code {
                    crate::code::anonymous::insert(module, index, old_unique, unique, arity, code);
                }

                arc_process
                    .anonymous_closure_with_env_from_slice(
                        module,
                        index,
                        old_unique,
                        unique,
                        arity,
                        option_code,
                        creator,
                        &[],
                    )
                    .unwrap()
            },
        )
        .boxed()
}

pub fn arity(arc_process: Arc<Process>) -> BoxedStrategy<Term> {
    arity_u8()
        .prop_map(move |u| arc_process.integer(u).unwrap())
        .boxed()
}

pub fn arity_or_arguments(arc_process: Arc<Process>) -> BoxedStrategy<Term> {
    prop_oneof![arity(arc_process.clone()), arguments(arc_process)].boxed()
}

pub fn arity_u8() -> BoxedStrategy<u8> {
    (0_u8..=255_u8).boxed()
}

pub fn arguments(arc_process: Arc<Process>) -> BoxedStrategy<Term> {
    super::list::proper(arc_process)
}

pub fn code(arc_process: &Arc<Process>) -> code::Result {
    arc_process.wait();

    Ok(())
}

pub fn export(arc_process: Arc<Process>) -> BoxedStrategy<Term> {
    (
        module_atom(),
        export::function(),
        arity_u8(),
        option_debuggable_code(),
    )
        .prop_map(move |(module, function, arity, option_debuggable_code)| {
            let option_code = option_debuggable_code.map(|debuggable_code| debuggable_code.0);

            if let Some(code) = option_code {
                crate::code::export::insert(module, function, arity, code);
            }

            arc_process
                .export_closure(module, function, arity, option_code)
                .unwrap()
        })
        .boxed()
}

pub fn is_not_arity_or_arguments(arc_process: Arc<Process>) -> BoxedStrategy<Term> {
    super::super::term(arc_process)
        .prop_filter("Arity and argument must be neither an arity (>= 0) or arguments (an empty or non-empty proper list)", |term| match term.to_typed_term().unwrap() {
            TypedTerm::Nil => false,
            TypedTerm::List(cons) => !cons.is_proper(),
            TypedTerm::Boxed(boxed) => match boxed.to_typed_term().unwrap() {
                TypedTerm::BigInteger(big_integer) => {
                    let big_int: &BigInt = big_integer.as_ref().into();
                    let zero_big_int: &BigInt = &0.into();

                    big_int < zero_big_int
                }
                _ => true
            }
            TypedTerm::SmallInteger(small_integer) => {
                let i: isize = small_integer.into();

                i < 0
            }
            _ => true,
        })
        .boxed()
}

pub fn option_debuggable_code() -> BoxedStrategy<Option<DebuggableCode>> {
    prop_oneof![Just(Some(DebuggableCode(code))), Just(None)].boxed()
}
