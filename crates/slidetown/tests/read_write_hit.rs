use slidetown::parsers::hit::Hit;
mod test_utils;
use test_utils::test_full_rewrite;

#[test]
fn mp_track1_area_hit() {
    test_full_rewrite::<Hit>("resources/hit/dcr_mp_track1_area.hit", (), ()).unwrap();
}
