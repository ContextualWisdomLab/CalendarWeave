//! Regression contract for the repository-local hosted-runner selector.

const TESTS_WORKFLOW: &str = include_str!("../.github/workflows/tests.yml");

#[test]
fn tests_workflow_uses_explicit_ubuntu_24_04_runners() {
    assert!(
        !TESTS_WORKFLOW.contains("runs-on: ubuntu-latest"),
        "floating ubuntu-latest can remain unassigned while explicit Ubuntu 24.04 executes"
    );

    let configured_jobs = TESTS_WORKFLOW.matches("runs-on:").count();
    let explicit_ubuntu_24_04_jobs = TESTS_WORKFLOW.matches("runs-on: ubuntu-24.04").count();

    assert!(
        configured_jobs >= 3,
        "Rust, coverage, and recovery jobs must all remain represented in the Tests workflow"
    );
    assert_eq!(
        explicit_ubuntu_24_04_jobs, configured_jobs,
        "every Tests workflow job must use the observed healthy explicit Ubuntu 24.04 image"
    );
}
