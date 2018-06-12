extern crate crossbeam;

#[macro_use]
extern crate uncover;

define_uncover_macros!(
    enable_if(true)
);

fn foo() {
    covered_by!("foo");
}

fn bar() { covered_by!("bar"); }

fn baz() {}

#[test]
fn test_covered() {
    covers!("bar");
    bar();
}

#[test]
#[should_panic]
fn test_not_covered() {
    covers!("bar");
    baz();
}

#[test]
fn no_multithreaded_false_positives() {
    crossbeam::scope(|scope| {
        for _ in 0..100 {
            scope.spawn(|| {
                for _ in 0..100 {
                    covers!("foo");
                    foo();
                }
            });
        }
    })
}
