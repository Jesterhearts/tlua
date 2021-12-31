use indoc::indoc;
use pretty_assertions::assert_eq;
use tlua::{
    compile,
    vm::runtime::{
        Gc,
        Runtime,
        Table,
    },
};

#[test]
#[ignore = "Tables are not fully implemented"]
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
