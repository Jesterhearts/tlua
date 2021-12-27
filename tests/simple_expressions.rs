use pretty_assertions::assert_eq;
use tlua::{
    compile,
    vm::runtime::Runtime,
};

#[test]
fn simple_addition() -> anyhow::Result<()> {
    let src = "return a + b";

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    rt.register_global("a", 1);
    rt.register_global("b", 2);

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![3.into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn simple_sequence() -> anyhow::Result<()> {
    let src = "
        x = a + 1
        y = b * 3
        return x - y
    ";

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    rt.register_global("a", 1);
    rt.register_global("b", 2);

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![(-4).into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn simple_reuse() -> anyhow::Result<()> {
    let src = "
        x = a + 1
        y = a * 3
        return x - y
    ";

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    rt.register_global("a", 1);

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![(-1).into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}
