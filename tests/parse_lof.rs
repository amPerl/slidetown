use slidetown::parsers::lof::Lof;
use std::io::Cursor;

#[test]
fn header_mp_modeltable0() {
    let lof_buffer = include_bytes!("../resources/lof/dev_mp_modeltable0_nodata.lof");
    let mut lof_cursor = Cursor::new(lof_buffer);
    let lof: Lof = Lof::parse(&mut lof_cursor).unwrap();

    assert_eq!(20061222, lof.header.version_date);
    assert_eq!(262, lof.header.model_count);

    assert_eq!(
        "미션_메테오_차3_랩2".to_string(),
        lof.models.last().unwrap().name
    );
}
