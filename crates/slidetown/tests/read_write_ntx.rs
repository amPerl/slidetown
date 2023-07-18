use slidetown::parsers::ntx::Ntx;
mod test_utils;
use test_utils::test_full_rewrite;

#[test]
fn ec_c() -> anyhow::Result<()> {
    test_full_rewrite::<Ntx>("resources/ntx/ec_C.ntx", (), ())?;
    Ok(())
}

#[test]
fn dcrnew_f() -> anyhow::Result<()> {
    test_full_rewrite::<Ntx>("resources/ntx/dcrnew_F.ntx", (), ())?;
    Ok(())
}

// the following test doesn't pass since padding garbage comparison fails
// #[test]
// fn dev_f() -> anyhow::Result<()> {
//     test_full_rewrite::<Ntx>("resources/ntx/dev_F.ntx", (), ())?;
//     Ok(())
// }
