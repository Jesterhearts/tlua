use indoc::indoc;
use pretty_assertions::assert_eq;
use tlua::{
    compile,
    vm::{
        runtime::Runtime,
        Value,
    },
};

#[test]
fn basic_va_args() -> anyhow::Result<()> {
    let src = indoc! {"
        local function foo(...)
            a, b, c, d = ...
            return a, b, c, d
        end

        return foo(1, 2, 3, 4)
    "};

    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![1.into(), 2.into(), 3.into(), 4.into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn partial_va_args() -> anyhow::Result<()> {
    let src = indoc! {"
        local function foo(...)
            a, b, c, d = ..., 6
            return a, b, c, d
        end

        return foo(1, 2, 3, 4)
    "};

    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![1.into(), 6.into(), Value::Nil, Value::Nil],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn va_arg_nested() -> anyhow::Result<()> {
    let src = indoc! {"
        local function foo(...)
            local function bar(a) return a end
            return bar(...)
        end

        return foo(1, 2, 3, 4)
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
fn va_args_middle_single_pop() -> anyhow::Result<()> {
    let src = indoc! {"
        local function foo(a, b, ...)
            local function bar(...) return ... end
            return bar(a, ..., b)
        end

        return foo(1, 2, 3, 4)
    "};

    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk)?;

    // ... in the middle only access the first va-arg in the list.
    assert_eq!(
        result,
        vec![1.into(), 3.into(), 2.into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}
