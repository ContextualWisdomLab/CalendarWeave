//! Regression contracts for repository-local hosted workflow queue discipline.

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

#[test]
fn database_jobs_use_an_explicit_loopback_address_for_the_operator() {
    assert_eq!(TESTS_WORKFLOW.matches("@127.0.0.1:5432/").count(), 2);
    assert!(!TESTS_WORKFLOW.contains("@localhost:5432/"));
}

#[test]
fn tests_workflow_cancels_superseded_exact_heads() {
    assert!(
        TESTS_WORKFLOW.contains(
            "group: calendarweave-tests-${{ github.event.pull_request.number || github.ref }}"
        ),
        "Tests must group runs by PR (or protected push ref) so a newer exact head supersedes older work"
    );
    assert!(
        TESTS_WORKFLOW.contains("cancel-in-progress: true"),
        "superseded Tests runs must release hosted-runner capacity instead of competing with the current head"
    );
}
