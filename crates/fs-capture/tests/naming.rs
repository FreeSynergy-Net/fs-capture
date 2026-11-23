use fs_capture::{capture_file_name, slug, MAX_SLUG_LEN};

#[test]
fn slug_keeps_simple_text() {
    assert_eq!(slug("Notepad - notes.txt"), "notepad_notes_txt");
}

#[test]
fn slug_removes_path_tricks() {
    for evil in [
        "..",
        "../../etc/passwd",
        "..\\..\\x",
        "/abs/path",
        "C:\\Windows\\x",
        "a/../b",
    ] {
        let s = slug(evil);
        assert!(
            !s.contains('.') && !s.contains('/') && !s.contains('\\') && !s.contains(':'),
            "{evil} -> {s}"
        );
    }
}

#[test]
fn slug_removes_windows_forbidden_chars() {
    let s = slug("a<b>c:d\"e/f\\g|h?i*j");
    assert_eq!(s, "a_b_c_d_e_f_g_h_i_j");
}

#[test]
fn slug_handles_reserved_names_in_any_case() {
    for r in ["CON", "nul", "Aux", "com1", "LPT9", "prn"] {
        let s = slug(r);
        assert!(s.ends_with('_'), "{r} -> {s}");
        assert_ne!(s.to_uppercase(), r.to_uppercase());
    }
    assert_eq!(slug("console"), "console");
}

#[test]
fn slug_limits_length_and_handles_empty() {
    assert!(slug(&"x".repeat(5000)).len() <= MAX_SLUG_LEN);
    assert_eq!(slug(""), "untitled");
    assert_eq!(slug("   "), "untitled");
    assert_eq!(slug("\u{4e2d}\u{6587}"), "untitled");
    assert_eq!(slug("..."), "untitled");
}

#[test]
fn slug_is_deterministic_and_trimmed() {
    assert_eq!(slug("--Hello  World--"), "hello_world");
    assert_eq!(slug("--Hello  World--"), slug("--Hello  World--"));
}

#[test]
fn file_name_has_number_time_kind() {
    let n = capture_file_name(7, "20261123T101500Z", "window", "../Evil");
    assert_eq!(n, "0007_20261123T101500Z_window_evil.png");
}
