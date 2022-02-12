use slidetown::parsers::lif::Lif;
mod test_utils;
use test_utils::test_full_rewrite;

#[test]
fn mp_track1_terrain0_lif() {
    test_full_rewrite::<Lif>("resources/lif/dcr_mp_track1_terrain0.lif", (), ()).unwrap();
}
