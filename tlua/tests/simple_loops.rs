use indoc::indoc;
use tlua::{
    compile,
    vm::runtime::Runtime,
};

#[test]
fn simple_while() -> anyhow::Result<()> {
    let src = indoc! {"
        local b = 0 
        while b < 10 do
            b = b + 1
        end

        return b
    "};
    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![10.into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn simple_while_break() -> anyhow::Result<()> {
    let src = indoc! {"
        local b = 0 
        while b < 10 do
            b = b + 1
            break
        end

        return b
    "};
    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![1.into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn simple_repeat() -> anyhow::Result<()> {
    let src = indoc! {"
        local b = 0 

        repeat
            b = b + 1
        until b == 10

        return b
    "};
    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![10.into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn simple_repeat_break() -> anyhow::Result<()> {
    let src = indoc! {"
        local b = 0 

        repeat
            b = b + 1
            break
        until b == 10

        return b
    "};
    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![1.into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}
