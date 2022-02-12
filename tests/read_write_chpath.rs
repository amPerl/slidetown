use slidetown::parsers::chpath::Chpath;
mod test_utils;
use test_utils::test_full_rewrite;

#[test]
fn oros_track1_chpath() {
    test_full_rewrite::<Chpath>("resources/chpath/PVP_map3_01.chpath", (), ()).unwrap();
}

#[test]
fn taipei_chpath() {
    test_full_rewrite::<Chpath>("resources/chpath/path_taipei.chpath", (), ()).unwrap();
}

#[test]
fn cras_chpath() {
    test_full_rewrite::<Chpath>("resources/chpath/path_Cras.chpath", (), ()).unwrap();
}
