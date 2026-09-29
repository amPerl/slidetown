use slidetown::parsers::hit::Hit;
mod test_utils;
use test_utils::test_full_rewrite;

#[test]
fn mp_track1_area_hit() {
    test_full_rewrite::<Hit>("resources/hit/dcr_mp_track1_area.hit", (), ()).unwrap();
}

#[test]
fn parkinglot_floor_p_hit_uppercase_magic() {
    let hit = test_full_rewrite::<Hit>("resources/hit/dcr_parkinglot_floor_p.hit", (), ()).unwrap();
    assert_eq!(&hit.header.magic, b"HIT\0");
    assert!(hit.verts.iter().all(|vert| vert.w.is_none()));
}

#[test]
fn koinonia_track8_floor_m_hit_20051005() {
    let hit =
        test_full_rewrite::<Hit>("resources/hit/dcr_koinonia_track8_floor_m.hit", (), ()).unwrap();
    assert_eq!(hit.header.version_date, 20051005);
    assert!(hit.verts.iter().all(|vert| vert.w == Some(0.0)));
}

#[test]
fn taipei_track3_floor_m_hit_20090629() {
    let hit =
        test_full_rewrite::<Hit>("resources/hit/dcr_taipei_track3_floor_m.hit", (), ()).unwrap();
    assert_eq!(hit.header.version_date, 20090629);
    assert!(hit.verts.iter().all(|vert| vert.w.is_some()));
}
