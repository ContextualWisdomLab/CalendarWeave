//! Regression contract for the repository-local hosted-runner selector.

const TESTS_WORKFLOW: &str = include_str!("../.github/workflows/tests.yml");

#[test]
fn tests_workflow_uses_explicit_ubuntu_24_04_runners() {
    assert!(
        !TESTS_WORKFLOW.contains("runs-on: ubuntu-latest"),
        "floating ubuntu-latest can remain unassigned while explicit Ubuntu 24.04 executes"
    );
    assert_eq!(
        TESTS_WORKFLOW.matches("runs-on: ubuntu-24.04").count(),
        2,
        "both Rust and coverage jobs must use the observed healthy hosted image"
    );
}
