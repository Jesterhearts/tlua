use indoc::indoc;
use pretty_assertions::assert_eq;
use tlua::{
    compile,
    vm::runtime::{
        value::table::TableKey,
        Gc,
        Runtime,
        Table,
        Value,
    },
};

#[test]
fn empty_table_init() -> anyhow::Result<()> {
    let src = indoc! {"
        local x = {}
        return x
    "};

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![Gc::new(Table::default()).into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn named_field_table_init() -> anyhow::Result<()> {
    let src = indoc! {r#"
        local x = { a = 11 }
        return x
    "#};

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    let result = rt.execute(&chunk)?;

    let mut expected = Table::default();
    expected
        .entries
        .insert(TableKey::try_from(Value::from("a")).unwrap(), 11.into());

    assert_eq!(
        result,
        vec![Gc::new(expected).into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn arraylike_field_table_init() -> anyhow::Result<()> {
    let src = indoc! {r#"
        local x = { 11 }
        return x
    "#};

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    let result = rt.execute(&chunk)?;

    let mut expected = Table::default();
    expected
        .entries
        .insert(TableKey::try_from(Value::from(1)).unwrap(), 11.into());

    assert_eq!(
        result,
        vec![Gc::new(expected).into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn indexed_field_table_init() -> anyhow::Result<()> {
    let src = indoc! {r#"
        local x = { ["a"] = 10 }
        return x
    "#};

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    let result = rt.execute(&chunk)?;

    let mut expected = Table::default();
    expected
        .entries
        .insert(TableKey::try_from(Value::from("a")).unwrap(), 10.into());

    assert_eq!(
        result,
        vec![Gc::new(expected).into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}
