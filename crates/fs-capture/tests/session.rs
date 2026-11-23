use std::fs;
use std::io::Read;

use fs_capture::{
    parse_index, CaptureSource, ErrorCode, FakeSource, FixedClock, ScreenId, Session, UtcTime,
    WindowId, INDEX_FILE,
};

fn clock() -> Box<FixedClock> {
    // 2026-11-23T10:15:00Z
    Box::new(FixedClock(UtcTime::from_unix(1_795_428_900)))
}

fn session(dir: &std::path::Path) -> Session {
    Session::open_with_clock(dir, clock()).unwrap()
}

fn png_files(dir: &std::path::Path) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".png"))
        .collect();
    v.sort();
    v
}

#[test]
fn add_list_remove() {
    let d = tempfile::tempdir().unwrap();
    let src = FakeSource::standard();
    let mut s = session(d.path());
    let e1 = s
        .add_screen(&src, src.primary_screen().unwrap().id)
        .unwrap();
    let e2 = s.add_window(&src, WindowId(1)).unwrap();
    assert_eq!((e1.nr, e2.nr), (1, 2));
    assert_eq!(e1.file, "0001_20261123T101500Z_screen_fake_1.png");
    assert_eq!(e2.app_name, "notepad.exe");
    assert_eq!(png_files(d.path()).len(), 2);

    // The list on disk matches memory, and a fresh open sees it.
    let reopened = session(d.path());
    assert_eq!(reopened.entries(), s.entries());
    let text = fs::read_to_string(d.path().join(INDEX_FILE)).unwrap();
    assert_eq!(parse_index(&text).unwrap(), s.entries());

    s.remove(1).unwrap();
    assert_eq!(s.entries().len(), 1);
    assert_eq!(png_files(d.path()), vec![e2.file.clone()]);
    // Numbers are not reused while a later one exists.
    let e3 = s.add_screen(&src, ScreenId(2)).unwrap();
    assert_eq!(e3.nr, 3);
}

#[test]
fn png_is_valid() {
    let d = tempfile::tempdir().unwrap();
    let src = FakeSource::standard();
    let mut s = session(d.path());
    let e = s.add_screen(&src, ScreenId(1)).unwrap();
    let bytes = fs::read(d.path().join(&e.file)).unwrap();
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    let dec = png::Decoder::new(std::io::Cursor::new(bytes));
    let mut r = dec.read_info().unwrap();
    let mut buf = vec![0; r.output_buffer_size().unwrap()];
    let info = r.next_frame(&mut buf).unwrap();
    assert_eq!((info.width, info.height), (8, 6));
    assert_eq!(&buf[..4], &[10, 1, 128, 255]);
}

#[test]
fn titles_with_tabs_and_newlines_survive_the_list() {
    let d = tempfile::tempdir().unwrap();
    let mut s = session(d.path());
    let img = fs_capture::Image::solid(2, 2, [1, 2, 3, 255]).unwrap();
    s.add_image(fs_capture::Kind::Window, "a\tb\nc\\d", "x\ty", &img)
        .unwrap();
    let again = session(d.path());
    assert_eq!(again.entries()[0].title, "a\tb\nc\\d");
    assert_eq!(again.entries()[0].app_name, "x\ty");
}

#[test]
fn error_cases() {
    let d = tempfile::tempdir().unwrap();
    let src = FakeSource::standard();
    let mut s = session(d.path());
    assert_eq!(
        s.add_screen(&src, ScreenId(99)).unwrap_err().code(),
        ErrorCode::UnknownScreen
    );
    assert_eq!(
        s.add_window(&src, WindowId(99)).unwrap_err().code(),
        ErrorCode::UnknownWindow
    );
    assert_eq!(
        s.add_window(&src, WindowId(3)).unwrap_err().code(),
        ErrorCode::WindowMinimized
    );
    assert_eq!(s.remove(1).unwrap_err().code(), ErrorCode::UnknownCapture);
    assert_eq!(
        src.window_by_title("notepad").unwrap_err().code(),
        ErrorCode::AmbiguousWindow
    );
    assert_eq!(
        src.window_by_title("nothing").unwrap_err().code(),
        ErrorCode::UnknownWindow
    );
    assert_eq!(src.window_by_title("TODO").unwrap().id, WindowId(2));
    assert!(s.entries().is_empty());
    assert!(png_files(d.path()).is_empty());
}

#[test]
fn folder_not_writable_is_an_error() {
    let d = tempfile::tempdir().unwrap();
    // A file where the folder should be: cannot be created or written.
    let blocker = d.path().join("file");
    fs::write(&blocker, b"x").unwrap();
    let err = Session::open(&blocker).err().unwrap();
    assert_eq!(err.code(), ErrorCode::Io);
    let err = Session::open(blocker.join("sub")).err().unwrap();
    assert_eq!(err.code(), ErrorCode::Io);
}

#[test]
fn damaged_list_is_reported() {
    let d = tempfile::tempdir().unwrap();
    fs::write(d.path().join(INDEX_FILE), "garbage\n").unwrap();
    assert_eq!(
        Session::open(d.path()).err().unwrap().code(),
        ErrorCode::BadIndex
    );
}

#[test]
fn zip_has_exactly_the_session_files_with_relative_names() {
    let d = tempfile::tempdir().unwrap();
    let src = FakeSource::standard();
    let mut s = session(d.path());
    s.add_screen(&src, ScreenId(1)).unwrap();
    s.add_window(&src, WindowId(2)).unwrap();
    // A stray file in the folder must not be bundled.
    fs::write(d.path().join("stray.txt"), b"secret").unwrap();
    let out = tempfile::tempdir().unwrap();
    let zip_path = out.path().join("a.zip");
    s.bundle(&zip_path).unwrap();

    let mut ar = zip::ZipArchive::new(fs::File::open(&zip_path).unwrap()).unwrap();
    let mut names: Vec<String> = (0..ar.len())
        .map(|i| ar.by_index(i).unwrap().name().unwrap().to_string())
        .collect();
    let mut expected: Vec<String> = s.entries().iter().map(|e| e.file.clone()).collect();
    expected.push(INDEX_FILE.to_string());
    names.sort();
    expected.sort();
    assert_eq!(names, expected);
    for n in &names {
        assert!(
            !n.contains('/') && !n.contains('\\') && !n.starts_with('.'),
            "{n}"
        );
    }
    let mut text = String::new();
    ar.by_name(INDEX_FILE)
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    assert_eq!(parse_index(&text).unwrap(), s.entries());
}

#[test]
fn zip_is_byte_identical_and_never_overwrites() {
    let d = tempfile::tempdir().unwrap();
    let src = FakeSource::standard();
    let mut s = session(d.path());
    s.add_screen(&src, ScreenId(1)).unwrap();
    s.add_window(&src, WindowId(1)).unwrap();
    let out = tempfile::tempdir().unwrap();
    let (a, b) = (out.path().join("a.zip"), out.path().join("b.zip"));
    s.bundle(&a).unwrap();
    // A later clock must not change the bytes.
    let s2 = Session::open(d.path()).unwrap();
    s2.bundle(&b).unwrap();
    assert_eq!(fs::read(&a).unwrap(), fs::read(&b).unwrap());
    assert_eq!(s.bundle(&a).unwrap_err().code(), ErrorCode::AlreadyExists);
}

#[cfg(unix)]
#[test]
fn zip_refuses_symlinked_capture_file() {
    let d = tempfile::tempdir().unwrap();
    let src = FakeSource::standard();
    let mut s = session(d.path());
    let e = s.add_screen(&src, ScreenId(1)).unwrap();
    let secret = d.path().join("outside.txt");
    fs::write(&secret, b"secret").unwrap();
    let p = d.path().join(&e.file);
    fs::remove_file(&p).unwrap();
    std::os::unix::fs::symlink(&secret, &p).unwrap();
    let out = tempfile::tempdir().unwrap();
    assert!(s.bundle(&out.path().join("x.zip")).is_err());
}

#[test]
fn time_formats() {
    assert_eq!(UtcTime::from_unix(0).to_rfc3339(), "1970-01-01T00:00:00Z");
    assert_eq!(
        UtcTime::from_unix(1_795_428_900).to_rfc3339(),
        "2026-11-23T10:15:00Z"
    );
    assert_eq!(
        UtcTime::from_unix(951_782_400).to_rfc3339(),
        "2000-02-29T00:00:00Z"
    );
}

#[cfg(unix)]
#[test]
fn read_only_folder_is_an_error_and_leaves_no_entry() {
    use std::os::unix::fs::PermissionsExt;
    let d = tempfile::tempdir().unwrap();
    let src = FakeSource::standard();
    let mut s = session(d.path());
    fs::set_permissions(d.path(), fs::Permissions::from_mode(0o555)).unwrap();
    let r = s.add_screen(&src, ScreenId(1));
    fs::set_permissions(d.path(), fs::Permissions::from_mode(0o755)).unwrap();
    // (As root the write would succeed; skip the check then.)
    if let Err(e) = r {
        assert_eq!(e.code(), ErrorCode::Io);
        assert!(s.entries().is_empty());
    }
}
