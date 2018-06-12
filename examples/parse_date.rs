#[macro_use]
extern crate uncover;

// This defines two macros, `covers!` and `covered_by!`.
// They will be no-ops unless `cfg!(debug_assertions)` is true.
define_uncover_macros!(
    enable_if(cfg!(debug_assertions))
);

pub fn parse_date(s: &str) -> Option<(u32, u32, u32)> {
    if 10 != s.len() {
        covered_by!("short date");
        return None;
    }

    if "-" != &s[4..5] || "-" != &s[7..8] {
        covered_by!("wrong dashes");
        return None;
    }

    let year = &s[0..4];
    let month = &s[6..7];
    let day = &s[8..10];

    year.parse::<u32>().ok().and_then(
        |y| month.parse::<u32>().ok().and_then(
            |m| day.parse::<u32>().ok().map(
                |d| (y, m, d))))
}


#[test]
fn test_parse_date() {
    {
        covers!("short date");
        assert!(parse_date("92").is_none());
    }

//  This will fail. Although the test looks like
//  it exercises the second condition, it does not.
//  The call to `covers!` call catches this bug in the test.
//  {
//      covers!("wrong dashes");
//      assert!(parse_date("8-26-1914").is_none());
//  }

    {
        covers!("wrong dashes");
        assert!(parse_date("19140-8-26").is_none());
    }
}
