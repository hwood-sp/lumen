use super::*;

#[test]
fn with_number_atom_reference_function_or_port_second_returns_second() {
    with_process_arc(|arc_process| {
        TestRunner::new(Config::with_source_file(file!()))
            .run(
                &(
                    strategy::term::pid::local(),
                    strategy::term::number_atom_reference_function_or_port(arc_process),
                ),
                |(first, second)| {
                    prop_assert_eq!(native(first, second), second);

                    Ok(())
                },
            )
            .unwrap();
    });
}

#[test]
fn with_lesser_local_pid_second_returns_second() {
    min(|_, _| make_pid(0, 0).unwrap(), Second);
}

#[test]
fn with_same_local_pid_second_returns_first() {
    min(|first, _| first, First);
}

#[test]
fn with_same_value_local_pid_second_returns_first() {
    min(|_, _| make_pid(0, 1).unwrap(), First);
}

#[test]
fn with_greater_local_pid_second_returns_first() {
    min(|_, _| make_pid(1, 1).unwrap(), First);
}

#[test]
fn with_external_pid_second_returns_first() {
    min(
        |_, process| process.external_pid(external_arc_node(), 2, 3).unwrap(),
        First,
    );
}

#[test]
fn with_list_or_bitstring_second_returns_first() {
    with_process_arc(|arc_process| {
        TestRunner::new(Config::with_source_file(file!()))
            .run(
                &(
                    strategy::term::pid::local(),
                    strategy::term::tuple_map_list_or_bitstring(arc_process),
                ),
                |(first, second)| {
                    prop_assert_eq!(native(first, second), first);

                    Ok(())
                },
            )
            .unwrap();
    });
}

fn min<R>(second: R, which: FirstSecond)
where
    R: FnOnce(Term, &Process) -> Term,
{
    super::min(|_| make_pid(0, 1).unwrap(), second, which);
}
