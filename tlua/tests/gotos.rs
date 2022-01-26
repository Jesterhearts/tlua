use indoc::indoc;
use tlua::{
    compile,
    vm::runtime::Runtime,
};

#[test]
fn basic_goto_forwards() -> anyhow::Result<()> {
    let src = indoc! {"
        local b = true

        goto a
        b = false
        ::a::

        return b
    "};
    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![true.into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn basic_goto_backwards() -> anyhow::Result<()> {
    let src = indoc! {"
        local b = true

        ::a::
        if b then
            b = false
            goto a
        end

        return b
    "};
    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![false.into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn goto_forwards_not_in_scope() -> anyhow::Result<()> {
    let src = indoc! {"
        local b = 1

        if b == 1 then
            goto a
            b = 2
        end

        ::a::
        if b == 1 then
            b = 3
        end

        return b
    "};
    let chunk = compile(src)?;

    let mut rt = Runtime::default();

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
fn goto_forwards_in_scope() -> anyhow::Result<()> {
    let src = indoc! {"
        local b = 1

        if b == 1 then
            goto a
            ::a::
            b = 2
        end

        ::a::
        if b == 1 then
            b = 3
        end

        return b
    "};
    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![2.into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}
