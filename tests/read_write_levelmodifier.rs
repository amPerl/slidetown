use slidetown::parsers::levelmodifier::*;
mod test_utils;
use test_utils::test_full_rewrite;

#[test]
fn dcr_levelmodifier() {
    test_full_rewrite::<LevelModifier>("resources/levelmodifier/dcr_levelmodifier.dat", (), ())
        .unwrap();
}

#[test]
fn dcrnew_levelmodifier() {
    test_full_rewrite::<LevelModifier>("resources/levelmodifier/dcrnew_levelmodifier.dat", (), ())
        .unwrap();
}

#[test]
fn dev_levelmodifier() {
    test_full_rewrite::<LevelModifier>("resources/levelmodifier/dev_levelmodifier.dat", (), ())
        .unwrap();
}
