// SIMPLE SCRIPTS
// ================================================================================================

#[test]
fn single_span() {
    let source = "begin push.1 push.2 add end";
    let program = super::compile_script(source).unwrap();
    let expected = "begin span push(1) push(2) add end end";
    assert_eq!(expected, format!("{}", program));
}

#[test]
fn span_and_simple_if() {
    // if with else
    let source = "begin push.1 push.2 if.true add else mul end end";
    let program = super::compile_script(source).unwrap();
    let expected = "\
        begin \
            join \
                span push(1) push(2) end \
                if.true span add end else span mul end end \
            end \
        end";
    assert_eq!(expected, format!("{}", program));

    // if without else
    let source = "begin push.1 push.2 if.true add end end";
    let program = super::compile_script(source).unwrap();
    let expected = "\
        begin \
            join \
                span push(1) push(2) end \
                if.true span add end else span noop end end \
            end \
        end";
    assert_eq!(expected, format!("{}", program));
}

// SCRIPTS WITH PROCEDURES
// ================================================================================================

#[test]
fn script_with_one_procedure() {
    let source = "proc.foo push.3 push.7 mul end begin push.1 push.2 add exec.foo end";
    let program = super::compile_script(source).unwrap();
    let expected = "begin span push(1) push(2) add end end";
    assert_eq!(expected, format!("{}", program));
}
