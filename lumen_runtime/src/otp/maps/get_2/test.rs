mod with_map;

use proptest::prop_assert_eq;
use proptest::strategy::{Just, Strategy};
use proptest::test_runner::{Config, TestRunner};

use liblumen_alloc::badmap;

use crate::otp::maps::get_2::native;
use crate::scheduler::with_process_arc;
use crate::test::strategy;

#[test]
fn without_map_errors_badmap() {
    with_process_arc(|arc_process| {
        TestRunner::new(Config::with_source_file(file!()))
            .run(
                &(
                    strategy::term(arc_process.clone()),
                    strategy::term::is_not_map(arc_process.clone()),
                ),
                |(key, map)| {
                    prop_assert_eq!(
                        native(&arc_process, key, map),
                        Err(badmap!(&arc_process, map))
                    );

                    Ok(())
                },
            )
            .unwrap();
    });
}
