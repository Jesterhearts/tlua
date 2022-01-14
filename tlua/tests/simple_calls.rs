use indoc::indoc;
use pretty_assertions::assert_eq;
use tlua::{
    compile,
    vm::runtime::{
        Runtime,
        Value,
    },
};

#[test]
fn local_call() -> anyhow::Result<()> {
    let src = indoc! {"
        local function foo() return bar end

        return foo()
    "};

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    rt.register_global("bar", 1);

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
fn local_call_with_arg() -> anyhow::Result<()> {
    let src = indoc! {"
        local function foo(a) return a end

        return foo(bar)
    "};

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    rt.register_global("bar", 1);

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
fn local_call_extra_arg_no_ret() -> anyhow::Result<()> {
    let src = indoc! {"
        local function foo() end

        return foo(10)
    "};

    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk)?;

    assert_eq!(result, vec![], "{:#?} produced an incorrect result", chunk);

    Ok(())
}

#[test]
fn local_call_multi_arg() -> anyhow::Result<()> {
    let src = indoc! {"
        local function foo(a, b) return b end

        return foo(10, 11)
    "};

    let chunk = compile(src)?;

    let mut rt = Runtime::default();

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
fn local_call_multi_arg_dup_name() -> anyhow::Result<()> {
    // Lua 5.4 allows multiple parameters to a function with the same name and only
    // uses the value for the last one.
    let src = indoc! {"
        local function foo(a, a) return a end

        return foo(10, 11)
    "};

    let chunk = compile(src)?;

    let mut rt = Runtime::default();

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
fn local_call_multi_ret() -> anyhow::Result<()> {
    let src = indoc! {"
        local function foo() return 10, 11 end

        x, y = foo()
        return y
    "};

    let chunk = compile(src)?;

    let mut rt = Runtime::default();

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
fn local_call_multi_ret_discards() -> anyhow::Result<()> {
    let src = indoc! {"
        local function foo() return 10, 11 end

        x = foo()
        return x
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
fn local_call_extra_arg_multi_ret_ignored() -> anyhow::Result<()> {
    let src = indoc! {"
        local function foo() return 10, 11 end

        x, y, z = foo(13)
        return z
    "};

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

#[test]
fn local_call_multi_ret_seq_pop1_popn() -> anyhow::Result<()> {
    let src = indoc! {"
        local function foo() return 10, 11 end

        x, y, z = foo(), foo()
        return x, y, z
    "};

    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk)?;

    // A call with multiple results in the middle of a multi-assign expression only
    // returns the first result.
    assert_eq!(
        result,
        vec![10.into(), 10.into(), 11.into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn local_call_multi_ret_seq_pop1_pop2() -> anyhow::Result<()> {
    let src = indoc! {"
        local function foo() return 10, 11 end

        x, y, z = foo(), 11, 12
        return x, y, z
    "};

    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![10.into(), 11.into(), 12.into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn local_call_call_as_arg() -> anyhow::Result<()> {
    let src = indoc! {"
        local function foo(...) return 10, 11, ... end

        x, y, z, w = foo(foo())
        return x, y, z, w
    "};

    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![10.into(), 11.into(), 10.into(), 11.into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}
