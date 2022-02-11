use slidetown::parsers::lf::Lf;
use std::io::Cursor;

#[test]
fn dcr_mp_terrain0_nodata_lf_rewrite() {
    let in_buf = include_bytes!("../resources/lf/dcr_mp_terrain0_nodata.lf");
    let mut in_file = Cursor::new(in_buf);
    let lf = Lf::read(&mut in_file).unwrap();

    assert_eq!(in_buf.len(), in_file.position() as usize);

    let mut out_buf = Vec::new();
    let mut out_file = Cursor::new(&mut out_buf);
    lf.write(&mut out_file).unwrap();

    for (i, (in_byte, out_byte)) in in_buf.iter().zip(out_buf.iter()).enumerate() {
        assert_eq!(
            in_byte, out_byte,
            "Mismatching byte at {}, in: {:02X}, out: {:02X}",
            i, in_byte, out_byte
        );
    }
}

#[test]
fn dev_mp_terrain0_nodata_lf_rewrite() {
    let in_buf = include_bytes!("../resources/lf/dev_mp_terrain0_nodata.lf");
    let mut in_file = Cursor::new(in_buf);
    let lf = Lf::read(&mut in_file).unwrap();

    assert_eq!(in_buf.len(), in_file.position() as usize);

    let mut out_buf = Vec::new();
    let mut out_file = Cursor::new(&mut out_buf);
    lf.write(&mut out_file).unwrap();

    for (i, (in_byte, out_byte)) in in_buf.iter().zip(out_buf.iter()).enumerate() {
        assert_eq!(
            in_byte, out_byte,
            "Mismatching byte at {}, in: {:02X}, out: {:02X}",
            i, in_byte, out_byte
        );
    }
}
