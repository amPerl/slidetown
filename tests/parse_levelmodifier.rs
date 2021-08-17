use slidetown::parsers::levelmodifier::*;
use std::io::Cursor;

#[test]
fn dcr_levelmodifier() {
    let lm_buffer = include_bytes!("../resources/fixtures/dcr_levelmodifier.dat");
    let mut lm_cursor = Cursor::new(lm_buffer);
    let lm: LevelModifier = LevelModifier::parse(&mut lm_cursor).unwrap();
    assert_eq!(lm.speed.len(), 801);
}

#[test]
fn dcrnew_levelmodifier() {
    let lm_buffer = include_bytes!("../resources/fixtures/dcrnew_levelmodifier.dat");
    let mut lm_cursor = Cursor::new(lm_buffer);
    let lm: LevelModifier = LevelModifier::parse(&mut lm_cursor).unwrap();
    assert_eq!(lm.speed.len(), 1001);
}
