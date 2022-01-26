use indoc::indoc;
use tlua::compile;
use tlua_compiler::CompileError;

#[test]
fn constant_cond_fails_multiple_labels() {
    let src = indoc! {"
        ::a::
        if false then
            ::a::
        end
    "};
    let result = compile(src);
    assert!(matches!(result, Err(CompileError::DuplicateLabel { .. })));
}

#[test]
#[ignore = "TODO: locals don't properly invalidate pending jumps."]
fn goto_across_local() {
    let src = indoc! {"
        goto a
        local b = 10

        ::a::

        return b
    "};
    let result = compile(src);
    assert!(matches!(result, Err(_)));
}
