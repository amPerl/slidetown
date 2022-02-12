use slidetown::parsers::loi::Loi;
mod test_utils;
use test_utils::test_full_rewrite;

#[test]
fn dcr_mp_main_object0_loi() {
    test_full_rewrite::<Loi>("resources/loi/dcr_mp_main_object0.loi", (3854,), ()).unwrap();
}

#[test]
fn dcr_mp_track1_object0_loi() {
    test_full_rewrite::<Loi>("resources/loi/dcr_mp_track1_object0.loi", (3854,), ()).unwrap();
}

#[test]
fn dev_mp_main_object0_loi() {
    test_full_rewrite::<Loi>("resources/loi/dev_mp_main_object0.loi", (3854,), ()).unwrap();
}

#[test]
fn dev_mp_track1_object0_loi() {
    test_full_rewrite::<Loi>("resources/loi/dev_mp_track1_object0.loi", (3854,), ()).unwrap();
}
