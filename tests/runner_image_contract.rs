//! Regression contracts for repository-local hosted workflow queue discipline.

const TESTS_WORKFLOW: &str = include_str!("../.github/workflows/tests.yml");

/// GitHub-hosted `ubuntu-*` labels are refused before a job starts while the
/// account is locked for billing. The isolated self-hosted group is the path
/// that actually executes the jobs.
#[test]
fn tests_workflow_uses_the_isolated_self_hosted_runner() {
    assert!(
        !TESTS_WORKFLOW.contains("ubuntu-24.04") && !TESTS_WORKFLOW.contains("ubuntu-latest"),
        "a GitHub-hosted image label is refused before the job starts"
    );
    let jobs = TESTS_WORKFLOW.matches("runs-on:").count();
    assert!(
        jobs >= 3,
        "Rust, coverage, and recovery jobs must all remain represented"
    );
    assert_eq!(
        TESTS_WORKFLOW.matches("group: CWL CI isolated").count(),
        jobs,
        "every job must target the isolated self-hosted runner group"
    );
    assert_eq!(
        TESTS_WORKFLOW.matches("calendarweave-ci").count(),
        jobs,
        "every job must use the CalendarWeave runner label so it cannot land on another repo's runner"
    );
}

/// No job may use the floating `ubuntu-latest` label, which can stay
/// unassigned while an explicit runner label executes. The jobs run on the
/// isolated self-hosted group (see
/// `tests_workflow_uses_the_isolated_self_hosted_runner`), not on a
/// GitHub-hosted image.
#[test]
fn tests_workflow_rejects_the_floating_ubuntu_latest_label() {
    assert!(
        !TESTS_WORKFLOW.contains("ubuntu-latest"),
        "floating ubuntu-latest can remain unassigned"
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
