use pretty_assertions::assert_eq;
use tlua::{
    compile,
    vm::runtime::Runtime,
};

#[test]
fn length() -> anyhow::Result<()> {
    let src = "return #a";

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    rt.register_global("a", "123");

    let result = rt.execute(&chunk);

    assert_eq!(
        result,
        Ok(vec![3.into()]),
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn constant_length() -> anyhow::Result<()> {
    let src = r#"return #"123""#;

    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk);

    assert_eq!(
        result,
        Ok(vec![3.into()]),
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn concat_strings() -> anyhow::Result<()> {
    let src = "return a..b";

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    rt.register_global("a", "foo");
    rt.register_global("b", "bar");

    let result = rt.execute(&chunk);

    assert_eq!(
        result,
        Ok(vec!["foobar".into()]),
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn constant_concat_strings() -> anyhow::Result<()> {
    let src = r#"return "foo".."bar" "#;

    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk);

    assert_eq!(
        result,
        Ok(vec!["foobar".into()]),
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn concat_string_number() -> anyhow::Result<()> {
    let src = "return a..b..c..d..e";

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    rt.register_global("a", "foo");
    rt.register_global("b", 2);
    rt.register_global("c", "bar");
    rt.register_global("d", 3.11);
    rt.register_global("e", "baz");

    let result = rt.execute(&chunk);

    assert_eq!(
        result,
        Ok(vec!["foo2bar3.11baz".into()]),
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn constant_concat_string_number() -> anyhow::Result<()> {
    let src = r#"return "foo"..(2).."bar"..(2.19).."baz" "#;

    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk);

    assert_eq!(
        result,
        Ok(vec!["foo2bar2.19baz".into()]),
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}
