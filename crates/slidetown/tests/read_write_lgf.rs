use slidetown::parsers::lgf::Lgf;
mod test_utils;
use test_utils::test_full_rewrite;

#[test]
fn dcr_mp_guardrail0_nodata_lgf_rewrite() {
    test_full_rewrite::<Lgf>("resources/lgf/dcr_mp_guardrail0_nodata.lgf", (), ()).unwrap();
}

#[test]
fn dev_mp_guardrail0_nodata_lgf_rewrite() {
    test_full_rewrite::<Lgf>("resources/lgf/dev_mp_guardrail0_nodata.lgf", (), ()).unwrap();
}
