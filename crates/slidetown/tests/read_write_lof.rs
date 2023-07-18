use slidetown::parsers::lof::Lof;
mod test_utils;
use test_utils::test_full_rewrite;

#[test]
fn dev_mp_modeltable0_nodata_lof_rewrite() {
    test_full_rewrite::<Lof>("resources/lof/dev_mp_modeltable0_nodata.lof", (), (None,)).unwrap();
}
