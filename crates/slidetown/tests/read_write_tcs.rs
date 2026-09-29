use slidetown::parsers::tcs::Tcs;
mod test_utils;
use test_utils::test_full_rewrite;

#[test]
fn oros_tcs() {
    test_full_rewrite::<Tcs>("resources/tcs/dcr_oros.tcs", (), ()).unwrap();
}
