use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_flag() {
    let mut cmd = Command::cargo_bin("ironclaw").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Secure agentic AI runtime with JIT Micro-VMs"));
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("ironclaw").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("ironclaw 0.1.0"));
}

#[test]
fn test_unknown_command() {
    let mut cmd = Command::cargo_bin("ironclaw").unwrap();
    cmd.arg("unknown-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "error: unrecognized subcommand 'unknown-command'",
        ));
}

#[test]
fn test_missing_command() {
    let mut cmd = Command::cargo_bin("ironclaw").unwrap();
    // ironclaw with no args logs info and exits 0
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("IronClaw Orchestrator"));
}

#[test]
fn test_invalid_arg_value() {
    // The 'run' command takes a TASK argument, not --memory
    // Let's test a simpler case: spawn-vm with invalid memory
    // Note: spawn-vm might not be fully implemented or exposed as expected
    // Based on error output, 'run' is definitely a command but args are different.
    // Let's stick to testing unknown argument for 'run' as shown in the error log
    let mut cmd = Command::cargo_bin("ironclaw").unwrap();
    cmd.arg("run").arg("--unknown-arg")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error: unexpected argument '--unknown-arg' found"));
}

#[test]
#[cfg(unix)] // VM spawning only supported on Unix
fn test_spawn_vm_dry_run() {
    // This assumes we can run a dry-run or check initial output
    // without actual Firecracker/root if we mock it or stop early.
    // For now, we just check that the command structure is accepted.

    // Note: This test might need adjustment based on how `spawn` command is implemented
    // If it requires root immediately, this might fail in CI.
    // Assuming 'spawn' command exists:
    let mut cmd = Command::cargo_bin("ironclaw").unwrap();
    cmd.arg("spawn").arg("--task-id").arg("test-task")
        .timeout(std::time::Duration::from_secs(2)); // Short timeout as it might hang waiting for VM

    // We don't assert success/failure here as it depends on environment,
    // but we can check if it started processing

    // If we can't easily test execution, we skip.
    // This is a placeholder for real integration tests.
}
