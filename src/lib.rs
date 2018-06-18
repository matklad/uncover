//! # uncover
//!
//! A library that makes tests easier to maintain by using instrumentation to
//! answer these two questions:
//!   * Which code is exercised by this test?
//!   * Which test covers this bit of code?
//!
//! Here's a short example:
//!
//! ```
//! #[macro_use]
//! extern crate uncover;
//!
//! // This defines two macros, `covers!` and `covered_by!`.
//! // They will be no-ops unless `cfg!(debug_assertions)` is true.
//! define_uncover_macros!(
//!     enable_if(cfg!(debug_assertions))
//! );
//!
//! fn parse_date(s: &str) -> Option<(u32, u32, u32)> {
//!     if 10 != s.len() {
//!         // By using `covered_by!("unique_name")`
//!         // we signal which test exercises this code.
//!         covered_by!("short date");
//!         return None;
//!     }
//!
//!     if "-" != &s[4..5] || "-" != &s[7..8] {
//!         covered_by!("wrong dashes");
//!         return None;
//!     }
//!     // ...
//!#    unimplemented!()
//! }
//!
//! #[test]
//! fn test_parse_date() {
//!     {
//!         // `covers!("unique_name")` creates a guard object
//!         // that verifies that by the end of the scope we've
//!         // executed the corresponding `covered_by!("unique_name")`.
//!         covers!("short date");
//!         assert!(parse_date("92").is_none());
//!     }
//!
//! //  This will fail. Although the test looks like
//! //  it exercises the second condition, it does not.
//! //  The call to `covers!` call catches this bug in the test.
//! //  {
//! //      covers!("wrong dashes");
//! //      assert!(parse_date("27.2.2013").is_none());
//! //  }
//!
//!     {
//!         covers!("wrong dashes");
//!         assert!(parse_date("27.02.2013").is_none());
//!     }
//! }
//!
//! # fn main() {}
//! ```
//!
//! ## Notes on concurrency
//!
//! Coverage is tracked via shared mutable state, so the following
//! caveat applies:
//!
//!   * A `covers!` from one test might be covered by thread of *another* test.
//!     As a result, a test might pass when it should have failed.
//!
//! The error in the opposite direction never happens: if your code covers everything
//! with a single thread, it will do it with several threads as well.

#[macro_use]
extern crate lazy_static;

use std::{
    sync::Mutex,
    collections::HashMap,
};


/// Define `covered_by!` and `covers!` macros.
///
/// Use `covered_by!("unique_name")` in the code and
/// `covers!("unique_name")` in the corresponding test to verify
/// that the test indeed covers the code. Under the hood,
/// `covers!` creates a guard object that checks coverage at scope
/// exit.
///
/// If called as `define_uncover_macros(enable_if(condition));`,
/// macros will be no-op unless condition is true. A typical condition
/// is
///
/// ```
/// # #[macro_use] extern crate uncover;
/// define_uncover_macros!(
///     enable_if(cfg!(debug_assertions))
/// );
/// ```
///
/// You can use condition to enable uncover based on compile-time env var:
///
/// ```
/// # #[macro_use] extern crate uncover;
/// define_uncover_macros!(
///     enable_if(option_env!("CI") == Some("1"))
/// );
/// ```
#[macro_export]
macro_rules! define_uncover_macros {
    (enable_if($cond:expr)) => {
        #[doc(hidden)]
        pub use $crate::__CoversGuard;
        #[doc(hidden)]
        pub use $crate::__covers_record_coverage;

        #[macro_export]
        macro_rules! covers {
            ($pos:expr) => {
                let mut _guard = None;
                if $cond {
                    _guard = Some($crate::__CoversGuard::new($pos))
                }
            };
        }

        #[macro_export]
        macro_rules! covered_by {
            ($pos:expr) => {
                if $cond {
                    $crate::__covers_record_coverage($pos);
                }
            };
        }
    };
}


lazy_static! {
    static ref STATE: Mutex<HashMap<&'static str, u64>> = Default::default();
}

#[doc(hidden)]
pub fn __covers_record_coverage(pos: &'static str) {
    *STATE.lock().unwrap().entry(pos).or_insert(0) += 1;
}

#[doc(hidden)]
pub fn __covers_get_coverage(pos: &'static str) -> u64 {
    *STATE.lock().unwrap().get(pos).unwrap_or(&0)
}


#[doc(hidden)]
pub struct __CoversGuard {
    pos: &'static str,
    cnt: u64,
}

impl __CoversGuard {
    #[doc(hidden)]
    pub fn new(pos: &'static str) -> __CoversGuard {
        let cnt = __covers_get_coverage(pos);
        __CoversGuard { pos, cnt }
    }
}

impl Drop for __CoversGuard {
    fn drop(&mut self) {
        if ::std::thread::panicking() {
            return;
        }
        if !(self.cnt < __covers_get_coverage(self.pos)) {
            panic!("not covered: {:?}", self.pos);
        }
    }
}
