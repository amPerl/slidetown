use slidetown::parsers::loi::Loi;
use std::io::Cursor;

#[test]
fn dcr_mp_main_object0_loi() {
    let in_buf = include_bytes!("../resources/loi/dcr_mp_main_object0.loi");
    let mut in_file = Cursor::new(in_buf);
    let loi = Loi::read(&mut in_file, 3854).unwrap();

    assert_eq!(in_buf.len(), in_file.position() as usize);

    let mut out_buf = Vec::new();
    let mut out_file = Cursor::new(&mut out_buf);
    loi.write(&mut out_file).unwrap();

    for (i, (in_byte, out_byte)) in in_buf.iter().zip(out_buf.iter()).enumerate() {
        assert_eq!(
            in_byte, out_byte,
            "Mismatching byte at {}, in: {:02X}, out: {:02X}",
            i, in_byte, out_byte
        );
    }
}

#[test]
fn dcr_mp_track1_object0_loi() {
    let in_buf = include_bytes!("../resources/loi/dcr_mp_track1_object0.loi");
    let mut in_file = Cursor::new(in_buf);
    let loi = Loi::read(&mut in_file, 3854).unwrap();

    assert_eq!(in_buf.len(), in_file.position() as usize);

    let mut out_buf = Vec::new();
    let mut out_file = Cursor::new(&mut out_buf);
    loi.write(&mut out_file).unwrap();

    for (i, (in_byte, out_byte)) in in_buf.iter().zip(out_buf.iter()).enumerate() {
        assert_eq!(
            in_byte, out_byte,
            "Mismatching byte at {}, in: {:02X}, out: {:02X}",
            i, in_byte, out_byte
        );
    }
}

#[test]
fn dev_mp_main_object0_loi() {
    let in_buf = include_bytes!("../resources/loi/dev_mp_main_object0.loi");
    let mut in_file = Cursor::new(in_buf);
    let loi = Loi::read(&mut in_file, 3854).unwrap();

    assert_eq!(in_buf.len(), in_file.position() as usize);

    let mut out_buf = Vec::new();
    let mut out_file = Cursor::new(&mut out_buf);
    loi.write(&mut out_file).unwrap();

    for (i, (in_byte, out_byte)) in in_buf.iter().zip(out_buf.iter()).enumerate() {
        assert_eq!(
            in_byte, out_byte,
            "Mismatching byte at {}, in: {:02X}, out: {:02X}",
            i, in_byte, out_byte
        );
    }
}

#[test]
fn dev_mp_track1_object0_loi() {
    let in_buf = include_bytes!("../resources/loi/dev_mp_track1_object0.loi");
    let mut in_file = Cursor::new(in_buf);
    let loi = Loi::read(&mut in_file, 3854).unwrap();

    assert_eq!(in_buf.len(), in_file.position() as usize);

    let mut out_buf = Vec::new();
    let mut out_file = Cursor::new(&mut out_buf);
    loi.write(&mut out_file).unwrap();

    for (i, (in_byte, out_byte)) in in_buf.iter().zip(out_buf.iter()).enumerate() {
        assert_eq!(
            in_byte, out_byte,
            "Mismatching byte at {}, in: {:02X}, out: {:02X}",
            i, in_byte, out_byte
        );
    }
}
