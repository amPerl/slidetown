use slidetown::parsers::lof::Lof;
use std::io::Cursor;

#[test]
fn header_mp_modeltable0() {
    let in_buf = include_bytes!("../resources/lof/dev_mp_modeltable0_nodata.lof");
    let mut in_file = Cursor::new(in_buf);
    let lof = Lof::read_without_data(&mut in_file).unwrap();

    assert_eq!(
        "미션_메테오_차3_랩2".to_string(),
        lof.models.last().unwrap().name
    );

    assert_eq!(in_buf.len(), in_file.position() as usize);

    let mut out_buf = Vec::new();
    let mut out_file = Cursor::new(&mut out_buf);
    lof.write_without_data(&mut out_file).unwrap();

    for (i, (in_byte, out_byte)) in in_buf.iter().zip(out_buf.iter()).enumerate() {
        assert_eq!(
            in_byte, out_byte,
            "Mismatching byte at {}, in: {:02X}, out: {:02X}",
            i, in_byte, out_byte
        );
    }
}
