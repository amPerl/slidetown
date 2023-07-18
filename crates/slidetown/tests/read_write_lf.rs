use slidetown::parsers::lf::Lf;
mod test_utils;
use test_utils::test_full_rewrite;

#[test]
fn dcr_mp_terrain0_nodata_lf_rewrite() {
    test_full_rewrite::<Lf>("resources/lf/dcr_mp_terrain0_nodata.lf", (), (None,)).unwrap();
}

#[test]
fn dev_mp_terrain0_nodata_lf_rewrite() {
    test_full_rewrite::<Lf>("resources/lf/dev_mp_terrain0_nodata.lf", (), (None,)).unwrap();
}
