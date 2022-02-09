use indoc::indoc;
use pretty_assertions::assert_eq;
use tlua::{
    compile,
    vm::runtime::Runtime,
};

#[test]
fn method_call() -> anyhow::Result<()> {
    let src = indoc! {r#"
        local t = { "a", "b", "c" }

        function t:index(i)
            return self[i]
        end

        return t:index(1)
    "#};
    let chunk = compile(src)?;

    eprintln!("{:#?}", chunk);

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk);

    assert_eq!(
        result,
        Ok(vec!["a".into()]),
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}
