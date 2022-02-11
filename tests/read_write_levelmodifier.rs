use slidetown::parsers::levelmodifier::*;
use std::io::Cursor;

#[test]
fn dcr_levelmodifier() {
    let in_buf = include_bytes!("../resources/levelmodifier/dcr_levelmodifier.dat");
    let mut in_file = Cursor::new(in_buf);
    let levelmodifier = LevelModifier::read(&mut in_file).unwrap();

    assert_eq!(in_buf.len(), in_file.position() as usize);

    let mut out_buf = Vec::new();
    let mut out_file = Cursor::new(&mut out_buf);
    levelmodifier.write(&mut out_file).unwrap();

    for (i, (in_byte, out_byte)) in in_buf.iter().zip(out_buf.iter()).enumerate() {
        assert_eq!(
            in_byte, out_byte,
            "Mismatching byte at {}, in: {:02X}, out: {:02X}",
            i, in_byte, out_byte
        );
    }
}

#[test]
fn dcrnew_levelmodifier() {
    let in_buf = include_bytes!("../resources/levelmodifier/dcrnew_levelmodifier.dat");
    let mut in_file = Cursor::new(in_buf);
    let levelmodifier = LevelModifier::read(&mut in_file).unwrap();

    assert_eq!(in_buf.len(), in_file.position() as usize);

    let mut out_buf = Vec::new();
    let mut out_file = Cursor::new(&mut out_buf);
    levelmodifier.write(&mut out_file).unwrap();

    for (i, (in_byte, out_byte)) in in_buf.iter().zip(out_buf.iter()).enumerate() {
        assert_eq!(
            in_byte, out_byte,
            "Mismatching byte at {}, in: {:02X}, out: {:02X}",
            i, in_byte, out_byte
        );
    }
}

#[test]
fn dev_levelmodifier() {
    let in_buf = include_bytes!("../resources/levelmodifier/dev_levelmodifier.dat");
    let mut in_file = Cursor::new(in_buf);
    let levelmodifier = LevelModifier::read(&mut in_file).unwrap();

    assert_eq!(in_buf.len(), in_file.position() as usize);

    let mut out_buf = Vec::new();
    let mut out_file = Cursor::new(&mut out_buf);
    levelmodifier.write(&mut out_file).unwrap();

    for (i, (in_byte, out_byte)) in in_buf.iter().zip(out_buf.iter()).enumerate() {
        assert_eq!(
            in_byte, out_byte,
            "Mismatching byte at {}, in: {:02X}, out: {:02X}",
            i, in_byte, out_byte
        );
    }
}
