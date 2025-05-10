//! Snapshot tests for LLVM IR codegen using insta
//! NOTE: codegen module not available, tests disabled

/*
use insta::assert_snapshot;
use wabznasm::codegen::dump_ir;

/// Simple addition IR snapshot (template-based)
#[test]
fn ir_snapshot_addition() {
    let ir = dump_ir("1+2").unwrap();
    assert_snapshot!(ir);
}

/// Factorial IR snapshot (template-based)
#[test]
fn ir_snapshot_factorial() {
    let ir = dump_ir("5!").unwrap();
    assert_snapshot!(ir);
}

/// Operator precedence IR snapshot (template-based)
#[test]
fn ir_snapshot_precedence_mul_add() {
    let ir = dump_ir("2*3+4").unwrap();
    assert_snapshot!(ir);
}
*/
