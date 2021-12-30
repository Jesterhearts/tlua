use pretty_assertions::assert_eq;
use tlua::{
    compile,
    vm::runtime::Runtime,
};

#[test]
fn simple_addition() -> anyhow::Result<()> {
    let src = "return a + b";

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    rt.register_global("a", 1);
    rt.register_global("b", 2);

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![3.into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn simple_sequence() -> anyhow::Result<()> {
    let src = "
        x = a + 1
        y = b * 3
        return x - y
    ";

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    rt.register_global("a", 1);
    rt.register_global("b", 2);

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![(-4).into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn simple_reuse() -> anyhow::Result<()> {
    let src = "
        x = a + 1
        y = a * 3
        return x - y
    ";

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    rt.register_global("a", 1);

    let result = rt.execute(&chunk)?;

    assert_eq!(
        result,
        vec![(-1).into()],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}

#[test]
fn all_basic_math() -> anyhow::Result<()> {
    let src = "
        add = a + 2
        sub = a - 2
        mul = a * 2
        div = a / 2
        idiv = a // 2
        mod = a % 2
        exp = a ^ 2
        uminus = -a
        band = a & 2
        bor = a | 2
        bxor = a ~ 2
        rsh = a >> 2
        lsh = a << 2
        ubnot = ~a
    ";

    let chunk = compile(src)?;

    let mut rt = Runtime::default();
    rt.register_global("a", 11.0);

    let _ = rt.execute(&chunk)?;

    let result = vec![
        rt.load_global("add").cloned(),
        rt.load_global("sub").cloned(),
        rt.load_global("mul").cloned(),
        rt.load_global("div").cloned(),
        rt.load_global("idiv").cloned(),
        rt.load_global("mod").cloned(),
        rt.load_global("exp").cloned(),
        rt.load_global("uminus").cloned(),
        rt.load_global("band").cloned(),
        rt.load_global("bor").cloned(),
        rt.load_global("bxor").cloned(),
        rt.load_global("rsh").cloned(),
        rt.load_global("lsh").cloned(),
        rt.load_global("ubnot").cloned(),
    ];

    assert_eq!(
        result,
        vec![
            // 11 + 2
            Some(13.0.into()),
            // 11 - 2
            Some(9.0.into()),
            // 11 * 2
            Some(22.0.into()),
            // 11 / 2
            Some(5.5.into()),
            // 11 // 2
            Some(5.into()),
            // 11 % 2
            Some(1.0.into()),
            // 11 ^ 2
            Some(121.0.into()),
            // -11
            Some((-11.0).into()),
            // 11 & 2
            Some(2.into()),
            // 11 | 2
            Some(11.into()),
            // 11 ~ 2
            Some(9.into()),
            // 11 >> 2
            Some(2.into()),
            // 11 << 2
            Some(44.into()),
            // ~11
            Some((-12).into()),
        ],
        "{:#?} produced an incorrect result",
        chunk
    );

    Ok(())
}
