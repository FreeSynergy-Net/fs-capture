//! Executes a command against a source; writes to the given output (testable).

use std::io::Write;

use fs_capture::{CaptureError, CaptureSource, ErrorCode, ScreenId, Session, WindowId};

use crate::args::{Command, Invocation, Target};

fn io(e: std::io::Error) -> CaptureError {
    CaptureError::new(ErrorCode::Io, e.to_string())
}

/// Runs one command. Output goes to `out`; errors are returned.
pub fn run(
    inv: &Invocation,
    source: &dyn CaptureSource,
    out: &mut dyn Write,
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
            let mut session = Session::open(&inv.dir)?;
            let entry = match target {
                Target::Primary => session.add_screen(source, source.primary_screen()?.id)?,
                Target::Screen(id) => session.add_screen(source, ScreenId(*id))?,
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
    use fs_capture::FakeSource;
    use std::path::PathBuf;

    fn go(command: Command, dir: &std::path::Path) -> Result<String, CaptureError> {
        let inv = Invocation {
            command,
            dir: PathBuf::from(dir),
        };
        let mut out = Vec::new();
        run(&inv, &FakeSource::standard(), &mut out)?;
        Ok(String::from_utf8_lossy(&out).into_owned())
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
