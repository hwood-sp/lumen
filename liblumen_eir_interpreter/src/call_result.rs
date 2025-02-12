use core::ptr::NonNull;

use std::convert::TryInto;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

use liblumen_alloc::borrow::clone_to_process::CloneToProcess;
use liblumen_alloc::erts::exception::runtime;
use liblumen_alloc::erts::process::code;
use liblumen_alloc::erts::process::{Process, Status};
use liblumen_alloc::erts::term::resource::Reference as ResourceReference;
use liblumen_alloc::erts::term::{Atom, Boxed, Closure, Term, TypedTerm};
use liblumen_alloc::erts::HeapFragment;

use lumen_runtime::process::spawn::options::Options;
use lumen_runtime::scheduler::{Scheduler, Spawned};
use lumen_runtime::system;

/// A sort of ghetto-future used to get the result from a process
/// spawn.
pub struct ProcessResultReceiver {
    pub process: Arc<Process>,
    rx: Receiver<ProcessResult>,
}

impl ProcessResultReceiver {
    pub fn try_get(&self) -> Option<ProcessResult> {
        self.rx.try_recv().ok()
    }
}

pub struct ProcessResult {
    pub heap: NonNull<HeapFragment>,
    pub result: Result<Term, (Term, Term, Term)>,
}

struct ProcessResultSender {
    tx: Sender<ProcessResult>,
}

pub fn call_run_erlang(
    proc: Arc<Process>,
    module: Atom,
    function: Atom,
    args: &[Term],
) -> ProcessResult {
    let recv = call_erlang(proc, module, function, args);
    let run_arc_process = recv.process.clone();

    loop {
        let ran = Scheduler::current().run_through(&run_arc_process);

        match *run_arc_process.status.read() {
            Status::Exiting(_) => {
                return recv.try_get().unwrap();
            }
            Status::Waiting => {
                if ran {
                    system::io::puts(&format!(
                        "WAITING Run queues len = {:?}",
                        Scheduler::current().run_queues_len()
                    ));
                } else {
                    panic!(
                        "{:?} did not run.  Deadlock likely in {:#?}",
                        run_arc_process,
                        Scheduler::current()
                    );
                }
            }
            Status::Runnable => {
                system::io::puts(&format!(
                    "RUNNABLE Run queues len = {:?}",
                    Scheduler::current().run_queues_len()
                ));
            }
            Status::Running => {
                system::io::puts(&format!(
                    "RUNNING Run queues len = {:?}",
                    Scheduler::current().run_queues_len()
                ));
            }
        }
    }
}

pub fn call_erlang(
    proc: Arc<Process>,
    module: Atom,
    function: Atom,
    args: &[Term],
) -> ProcessResultReceiver {
    let (tx, rx) = channel();

    let sender = ProcessResultSender { tx };
    let sender_term = proc.resource(Box::new(sender)).unwrap();

    let return_ok = {
        let module = Atom::try_from_str("lumen_eir_interpreter_intrinsics").unwrap();
        const ARITY: u8 = 1;

        proc.anonymous_closure_with_env_from_slice(
            module,
            // TODO assign `index` scoped to `module`
            0,
            // TODO calculate `old_unique` for `return_ok` with `sender_term` captured.
            Default::default(),
            // TODO calculate `unique` for `return_ok` with `sender_term` captured.
            Default::default(),
            ARITY,
            Some(return_ok),
            proc.pid().into(),
            &[sender_term],
        )
        .unwrap()
    };

    let return_throw = {
        let module = Atom::try_from_str("lumen_eir_interpreter_intrinsics").unwrap();
        const ARITY: u8 = 1;

        proc.anonymous_closure_with_env_from_slice(
            module,
            // TODO assing `index` scoped to `module`
            1,
            // TODO calculate `unique` for `return_throw` with `sender_term` captured.
            Default::default(),
            // TODO calculate `unique` for `return_throw` with `sender_term` captured.
            Default::default(),
            ARITY,
            Some(return_throw),
            proc.pid().into(),
            &[sender_term],
        )
        .unwrap()
    };

    let mut args_vec = vec![return_ok, return_throw];
    args_vec.extend(args.iter().cloned());

    let arguments = proc.list_from_slice(&args_vec).unwrap();

    let options: Options = Default::default();
    //options.min_heap_size = Some(100_000);

    let Spawned {
        arc_process: run_arc_process,
        ..
    } = Scheduler::spawn_apply_3(&proc, options, module, function, arguments).unwrap();

    ProcessResultReceiver {
        process: run_arc_process,
        rx,
    }
}

fn return_ok(arc_process: &Arc<Process>) -> code::Result {
    let argument_list = arc_process.stack_pop().unwrap();
    let closure_term = arc_process.stack_pop().unwrap();

    let mut argument_vec: Vec<Term> = Vec::new();
    match argument_list.to_typed_term().unwrap() {
        TypedTerm::Nil => (),
        TypedTerm::List(argument_cons) => {
            for result in argument_cons.into_iter() {
                let element = result.unwrap();

                argument_vec.push(element);
            }
        }
        _ => panic!(),
    }
    assert!(argument_vec.len() == 1);

    let closure: Boxed<Closure> = closure_term.try_into().unwrap();
    let sender_any: ResourceReference = closure.env_slice()[0].try_into().unwrap();
    let sender: &ProcessResultSender = sender_any.downcast_ref().unwrap();

    let mut fragment = unsafe { HeapFragment::new_from_word_size(100) }.unwrap();
    let frag_mut = unsafe { fragment.as_mut() };
    let ret = argument_vec[0].clone_to_heap(frag_mut).unwrap();

    sender
        .tx
        .send(ProcessResult {
            heap: fragment,
            result: Ok(ret),
        })
        .unwrap();

    Ok(arc_process.return_from_call(argument_vec[0])?)
}

fn return_throw(arc_process: &Arc<Process>) -> code::Result {
    let argument_list = arc_process.stack_pop().unwrap();
    let closure_term = arc_process.stack_pop().unwrap();

    let mut argument_vec: Vec<Term> = Vec::new();
    match argument_list.to_typed_term().unwrap() {
        TypedTerm::Nil => (),
        TypedTerm::List(argument_cons) => {
            for result in argument_cons.into_iter() {
                let element = result.unwrap();

                argument_vec.push(element);
            }
        }
        _ => panic!(),
    }

    let closure: Boxed<Closure> = closure_term.try_into().unwrap();
    let sender_any: ResourceReference = closure.env_slice()[0].try_into().unwrap();
    let sender: &ProcessResultSender = sender_any.downcast_ref().unwrap();

    let mut fragment = unsafe { HeapFragment::new_from_word_size(100) }.unwrap();
    let frag_mut = unsafe { fragment.as_mut() };

    let ret_type = argument_vec[0].clone_to_heap(frag_mut).unwrap();
    let ret_reason = argument_vec[1].clone_to_heap(frag_mut).unwrap();
    let ret_trace = argument_vec[2].clone_to_heap(frag_mut).unwrap();

    sender
        .tx
        .send(ProcessResult {
            heap: fragment,
            result: Err((ret_type, ret_reason, ret_trace)),
        })
        .unwrap();

    let class: Atom = argument_vec[0].try_into().unwrap();
    let class = match class.name() {
        "EXIT" => runtime::Class::Exit,
        "throw" => runtime::Class::Throw,
        "error" => runtime::Class::Error { arguments: None },
        k => unreachable!("{:?}", k),
    };

    let exc = runtime::Exception {
        class,
        reason: argument_vec[1],
        stacktrace: Some(argument_vec[2]),
        file: "",
        line: 0,
        column: 0,
    };
    code::result_from_exception(arc_process, exc.into())
}
