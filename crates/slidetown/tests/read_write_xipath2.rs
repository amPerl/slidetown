use slidetown::parsers::xipath2::Xipath2;
mod test_utils;
use test_utils::test_full_rewrite;

#[test]
fn cras_huv00_xipath2() {
    test_full_rewrite::<Xipath2>("resources/xipath2/dcr_cras_huv00.xipath2", (), ()).unwrap();
}

#[test]
fn koinonia_huv00_xipath2() {
    test_full_rewrite::<Xipath2>("resources/xipath2/dcr_koinonia_huv00.xipath2", (), ()).unwrap();
}

#[test]
fn moonpalace_huv00_xipath2() {
    test_full_rewrite::<Xipath2>("resources/xipath2/dcr_moonpalace_huv00.xipath2", (), ()).unwrap();
}

#[test]
fn oros_huv00_xipath2() {
    test_full_rewrite::<Xipath2>("resources/xipath2/dcr_oros_huv00.xipath2", (), ()).unwrap();
}
