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

#[test]
fn va_table_init() -> anyhow::Result<()> {
    let src = indoc! {r#"
        local function foo(...)
            local x = { ..., ... }
            return x
        end
        return foo(1, 2, 3)
    "#};

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    let result = rt.execute(&chunk)?;

    let mut expected = Table::default();
    expected.entries.extend([
        (TableKey::try_from(Value::from(1)).unwrap(), 1.into()),
        (TableKey::try_from(Value::from(2)).unwrap(), 1.into()),
        (TableKey::try_from(Value::from(3)).unwrap(), 2.into()),
        (TableKey::try_from(Value::from(4)).unwrap(), 3.into()),
    ]);

    assert_eq!(result.len(), 1, "{:#?} produced an incorrect result", chunk);

    assert!(matches!(result.first(), Some(Value::Table(_))));

    if let Some(Value::Table(t)) = result.first() {
        assert_eq!(
            t.borrow().entries,
            expected.entries,
            "{:#?} produced an incorrect result",
            chunk
        );
    }

    Ok(())
}

#[test]
fn arraylike_takes_precedence_table_init() -> anyhow::Result<()> {
    let src = indoc! {r#"
        local x = { [1] = 13, 10, 11 }
        local y = { 10, [1] = 13, 11 }
        return x, y
    "#};

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    let result = rt.execute(&chunk)?;

    let mut expected = Table::default();
    expected.entries.extend([
        (TableKey::try_from(Value::from(1)).unwrap(), 10.into()),
        (TableKey::try_from(Value::from(2)).unwrap(), 11.into()),
    ]);

    assert_eq!(result.len(), 2, "{:#?} produced an incorrect result", chunk);

    assert!(matches!(
        result.as_slice(),
        [Value::Table(_), Value::Table(_)]
    ));

    if let [Value::Table(x), Value::Table(y)] = result.as_slice() {
        assert_eq!(
            x.borrow().entries,
            expected.entries,
            "{:#?} produced an incorrect result",
            chunk
        );

        assert_eq!(
            y.borrow().entries,
            expected.entries,
            "{:#?} produced an incorrect result",
            chunk
        );
    }

    Ok(())
}
