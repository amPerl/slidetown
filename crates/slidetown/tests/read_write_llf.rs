use slidetown::parsers::llf::Llf;
mod test_utils;
use test_utils::test_full_rewrite;

#[test]
fn dcr_mp_lane0_nodata_llf_rewrite() {
    test_full_rewrite::<Llf>("resources/llf/dcr_mp_lane0_nodata.llf", (), ()).unwrap();
}

#[test]
fn dev_mp_lane0_nodata_llf_rewrite() {
    test_full_rewrite::<Llf>("resources/llf/dev_mp_lane0_nodata.llf", (), ()).unwrap();
}
