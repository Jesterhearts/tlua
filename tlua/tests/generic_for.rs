use indoc::indoc;
use pretty_assertions::assert_eq;
use tlua::{
    compile,
    vm::runtime::Runtime,
};

#[test]
fn generic_for() -> anyhow::Result<()> {
    let src = indoc! {r#"
        local b = 777 
        local t = { "a", "b", "c" }

        local function next(state, control)
            control = control or 0
            control = control + 1
            if state[control] then
                return control, state[control]
            end
        end

        for k,v in next,t,nil do
            b = v
        end

        return b
    "#};
    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk);

    assert_eq!(
        result,
        Ok(vec!["c".into()]),
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn generic_for_break() -> anyhow::Result<()> {
    let src = indoc! {r#"
        local b = 0 
        local t = { "a", "b", "c" }

        local function next(state, control)
            control = control or 0
            control = control + 1
            if state[control] then
                return control, state[control]
            end
        end

        for k,v in next,t,nil do
            b = v
            break
        end

        return b
    "#};
    let chunk = compile(src)?;

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
