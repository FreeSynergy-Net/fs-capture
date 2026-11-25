//! The real binary: a whole screen is refused without `--full-screen` (no display needed, because
//! the refusal happens before any screen is touched).

use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_fs-capture"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn whole_screen_without_flag_exits_1_with_stable_code() {
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join("s");
    for target in [&["--primary"][..], &["--screen", "1"][..]] {
        let mut a = vec!["shot"];
        a.extend_from_slice(target);
        a.extend_from_slice(&["--dir", dir.to_str().unwrap()]);
        let o = run(&a);
        assert_eq!(o.status.code(), Some(1));
        let err = String::from_utf8_lossy(&o.stderr);
        assert!(err.contains("error[E_FULL_SCREEN_NOT_CONFIRMED]"), "{err}");
        assert!(o.stdout.is_empty());
    }
    assert!(!dir.exists(), "nothing is created when refused");
}

#[test]
fn flag_with_a_command_that_takes_no_screen_is_a_usage_error() {
    let o = run(&["list", "--full-screen"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("E_USAGE"));
}
