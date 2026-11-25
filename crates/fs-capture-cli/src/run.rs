//! Executes a command against a source; writes to the given output (testable).

use std::io::Write;

use fs_capture::{
    CaptureError, CaptureSource, ErrorCode, ScreenId, Session, WindowId, FULL_SCREEN_WARNING,
};

use crate::args::{Command, Invocation, Target};

fn io(e: std::io::Error) -> CaptureError {
    CaptureError::new(ErrorCode::Io, e.to_string())
}

/// Runs one command. Output goes to `out`, warnings to `warn`; errors are returned.
pub fn run(
    inv: &Invocation,
    source: &dyn CaptureSource,
    out: &mut dyn Write,
    warn: &mut dyn Write,
) -> Result<(), CaptureError> {
    match &inv.command {
        Command::Help => out.write_all(crate::args::HELP.as_bytes()).map_err(io),
        Command::Screens => {
            for s in source.screens()? {
                let p = if s.primary { "primary" } else { "-" };
                writeln!(
                    out,
                    "{}\t{}\t{}x{}\tscale {:.2}\t{}",
                    s.id.0, s.name, s.width, s.height, s.scale, p
                )
                .map_err(io)?;
            }
            Ok(())
        }
        Command::Windows => {
            for w in source.windows()? {
                let m = if w.minimized { "minimized" } else { "-" };
                writeln!(
                    out,
                    "{}\t{}\t{}\t{}x{}\t{}",
                    w.id.0, w.title, w.app_name, w.width, w.height, m
                )
                .map_err(io)?;
            }
            Ok(())
        }
        Command::Shot(target) => {
            let whole_screen = matches!(target, Target::Primary | Target::Screen(_));
            if whole_screen {
                // Refuse before anything else is looked up or opened.
                inv.full_screen.require()?;
                writeln!(warn, "warning: {FULL_SCREEN_WARNING}").map_err(io)?;
            }
            let mut session = Session::open(&inv.dir)?;
            let consent = inv.full_screen;
            let entry = match target {
                Target::Primary => {
                    session.add_screen(source, source.primary_screen()?.id, consent)?
                }
                Target::Screen(id) => session.add_screen(source, ScreenId(*id), consent)?,
                Target::Window(id) => session.add_window(source, WindowId(*id))?,
                Target::Title(t) => session.add_window(source, source.window_by_title(t)?.id)?,
            };
            writeln!(out, "{}\t{}", entry.nr, entry.file).map_err(io)
        }
        Command::List => {
            let session = Session::open(&inv.dir)?;
            for e in session.entries() {
                writeln!(
                    out,
                    "{}\t{}\t{}\t{}\t{}\t{}x{}",
                    e.nr,
                    e.time.to_rfc3339(),
                    e.kind.as_str(),
                    e.title,
                    e.app_name,
                    e.width,
                    e.height
                )
                .map_err(io)?;
            }
            Ok(())
        }
        Command::Delete(nr) => {
            let mut session = Session::open(&inv.dir)?;
            let e = session.remove(*nr)?;
            writeln!(out, "deleted {}\t{}", e.nr, e.file).map_err(io)
        }
        Command::Zip(file) => {
            Session::open(&inv.dir)?.bundle(file)?;
            writeln!(out, "wrote {}", file.display()).map_err(io)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fs_capture::{FakeSource, FullScreenConsent};
    use std::path::PathBuf;

    fn go_with(
        command: Command,
        dir: &std::path::Path,
        full_screen: FullScreenConsent,
    ) -> Result<(String, String), CaptureError> {
        let inv = Invocation {
            command,
            dir: PathBuf::from(dir),
            full_screen,
        };
        let (mut out, mut warn) = (Vec::new(), Vec::new());
        run(&inv, &FakeSource::standard(), &mut out, &mut warn)?;
        Ok((
            String::from_utf8_lossy(&out).into_owned(),
            String::from_utf8_lossy(&warn).into_owned(),
        ))
    }

    /// With consent (the old tests are about the flow, not the guard).
    fn go(command: Command, dir: &std::path::Path) -> Result<String, CaptureError> {
        go_with(command, dir, FullScreenConsent::Confirmed).map(|(o, _)| o)
    }

    #[test]
    fn whole_screen_needs_the_flag() {
        let d = tempfile::tempdir().unwrap();
        let s = d.path().join("session");
        for t in [Target::Primary, Target::Screen(2)] {
            let e = go_with(Command::Shot(t), &s, FullScreenConsent::Denied).unwrap_err();
            assert_eq!(e.code(), ErrorCode::FullScreenNotConfirmed);
        }
        assert!(!s.exists(), "nothing is created when refused");
        let (out, warn) = go_with(
            Command::Shot(Target::Primary),
            &s,
            FullScreenConsent::Confirmed,
        )
        .unwrap();
        assert!(out.starts_with("1\t"));
        assert!(warn.contains("whole screen") && warn.contains("Bildschirm"));
    }

    #[test]
    fn window_needs_no_flag_and_gives_no_warning() {
        let d = tempfile::tempdir().unwrap();
        let s = d.path().join("session");
        let (out, warn) = go_with(
            Command::Shot(Target::Title("todo".into())),
            &s,
            FullScreenConsent::Denied,
        )
        .unwrap();
        assert!(out.starts_with("1\t"));
        assert!(warn.is_empty());
    }

    #[test]
    fn shot_list_delete_zip_flow() {
        let d = tempfile::tempdir().unwrap();
        let s = d.path().join("session");
        assert!(go(Command::Screens, &s).unwrap().contains("primary"));
        go(Command::Shot(Target::Primary), &s).unwrap();
        go(Command::Shot(Target::Title("todo".into())), &s).unwrap();
        assert_eq!(go(Command::List, &s).unwrap().lines().count(), 2);
        go(Command::Delete(1), &s).unwrap();
        let zip = d.path().join("out.zip");
        go(Command::Zip(zip.clone()), &s).unwrap();
        assert!(zip.is_file());
        let err = go(Command::Zip(zip), &s).unwrap_err();
        assert_eq!(err.code(), ErrorCode::AlreadyExists);
    }

    #[test]
    fn errors_carry_codes() {
        let d = tempfile::tempdir().unwrap();
        let s = d.path().join("session");
        let c = |t| go(Command::Shot(t), &s).unwrap_err().code();
        assert_eq!(c(Target::Screen(99)), ErrorCode::UnknownScreen);
        assert_eq!(c(Target::Window(99)), ErrorCode::UnknownWindow);
        assert_eq!(c(Target::Window(3)), ErrorCode::WindowMinimized);
        assert_eq!(
            c(Target::Title("notepad".into())),
            ErrorCode::AmbiguousWindow
        );
        assert_eq!(
            go(Command::Delete(5), &s).unwrap_err().code(),
            ErrorCode::UnknownCapture
        );
    }
}
