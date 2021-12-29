use indoc::indoc;
use pretty_assertions::assert_eq;
use tlua::{
    compile,
    vm::runtime::{
        Runtime,
        Value,
    },
};

const SIMPLE_IF: &str = indoc! {"
    if a then
        return 10
    end

    return 11
"};

#[test]
fn simple_if_true() -> anyhow::Result<()> {
    let src = SIMPLE_IF;
    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    rt.register_global("a", true);

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
fn simple_if_false() -> anyhow::Result<()> {
    let src = SIMPLE_IF;
    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    rt.register_global("a", false);

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![11.into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

const IF_ELSE: &str = indoc! {"
    local b
    if a then
        b = 10
    else
        b = 11
    end

    return b
"};

#[test]
fn if_else_true() -> anyhow::Result<()> {
    let src = IF_ELSE;
    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    rt.register_global("a", true);

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
fn if_else_false() -> anyhow::Result<()> {
    let src = IF_ELSE;

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    rt.register_global("a", false);

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![11.into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

const IF_ELIF_CHAIN: &str = indoc! {"
    local x
    if a then
        x = 10
    elseif b then
        x = 11
    elseif c then
        x = 12
    end

    return x
"};

#[test]
fn if_elif_chain0() -> anyhow::Result<()> {
    let src = IF_ELIF_CHAIN;

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    rt.register_global("a", true);

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
fn if_elif_chain1() -> anyhow::Result<()> {
    let src = IF_ELIF_CHAIN;

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    rt.register_global("a", false);
    rt.register_global("b", true);

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![11.into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn if_elif_chain2() -> anyhow::Result<()> {
    let src = IF_ELIF_CHAIN;

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    rt.register_global("a", false);
    rt.register_global("b", false);
    rt.register_global("c", true);

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![12.into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn if_elif_chain_none() -> anyhow::Result<()> {
    let src = IF_ELIF_CHAIN;

    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![Value::Nil],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

const IF_ELIF_ELSE: &str = indoc! {"
    local x
    if a then
        x = 10
    elseif b then
        x = 11
    else
        x = 13
    end

    return x
"};

#[test]
fn if_elif_else() -> anyhow::Result<()> {
    let src = IF_ELIF_ELSE;

    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![13.into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}
