#[test]
fn test_command() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/command.rs");
    t.pass("tests/ui/command_generic.rs");
}
