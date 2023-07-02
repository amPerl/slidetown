use slidetown::parsers::tdf::Tdf;
mod test_utils;
use test_utils::test_full_rewrite;

#[test]
fn tutorial_2009() -> anyhow::Result<()> {
    test_full_rewrite::<Tdf>("resources/tdf/2009_Tutorial.tdf", (), ())?;
    Ok(())
}

#[test]
fn tutorial_dev() -> anyhow::Result<()> {
    test_full_rewrite::<Tdf>("resources/tdf/dev_Tutorial.tdf", (), ())?;
    Ok(())
}
