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
fn nested_function_decl_always_new_captures() -> anyhow::Result<()> {
    // The behavior here is identical if bar is global.
    let src = indoc! {"
        local function foo(a)
            local b
            local function bar() return b end
            if a then
                b = 10
                return foo(false)
            else
                return bar()
            end
        end
    
        return foo(true)
    "};

    let chunk = compile(src)?;

    let mut rt = Runtime::default();

    let result = rt.execute(&chunk);

    assert_eq!(
        result,
        Ok(vec![Value::Nil]),
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}
