//! Command line parsing (no external parser: the grammar is tiny).

use std::path::PathBuf;

/// What to capture.
#[derive(Debug, PartialEq, Eq)]
pub enum Target {
    /// `--primary`
    Primary,
    /// `--screen <id>`
    Screen(u64),
    /// `--window <id>`
    Window(u64),
    /// `--title <text>`
    Title(String),
}

/// A parsed command.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// `screens`
    Screens,
    /// `windows`
    Windows,
    /// `shot ...`
    Shot(Target),
    /// `list`
    List,
    /// `delete <nr>`
    Delete(u32),
    /// `zip <file.zip>`
    Zip(PathBuf),
    /// `--help`
    Help,
}

/// Command plus the session folder.
#[derive(Debug, PartialEq, Eq)]
pub struct Invocation {
    /// The command.
    pub command: Command,
    /// Session folder (`--dir`, default `captures`).
    pub dir: PathBuf,
}

/// Help text.
pub const HELP: &str = "\
fs-capture - screenshots of screens and windows, collected in a folder, bundled as ZIP

USAGE:
  fs-capture screens                      list screens
  fs-capture windows                      list windows
  fs-capture shot --primary               capture the primary screen
  fs-capture shot --screen <id>           capture one screen
  fs-capture shot --window <id>           capture one window
  fs-capture shot --title <text>          capture the one window whose title contains <text>
  fs-capture list                         list the captures in the folder
  fs-capture delete <nr>                  delete one capture
  fs-capture zip <file.zip>               bundle all captures (an existing file is never overwritten)

OPTIONS:
  --dir <folder>   session folder (default: ./captures)
  -h, --help       show this help

Errors are printed as `error[E_CODE]: message` on stderr; exit code 1 (2 = wrong usage).
";

/// Parses the arguments (without the program name). `Err` is a usage message.
pub fn parse(args: &[String]) -> Result<Invocation, String> {
    let mut dir = PathBuf::from("captures");
    let mut rest: Vec<&str> = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--dir" => {
                dir = PathBuf::from(it.next().ok_or("--dir needs a folder")?);
            }
            "-h" | "--help" => rest.insert(0, "--help"),
            other => rest.push(other),
        }
    }
    let command = match rest.as_slice() {
        ["--help", ..] | [] => Command::Help,
        ["screens"] => Command::Screens,
        ["windows"] => Command::Windows,
        ["list"] => Command::List,
        ["shot", "--primary"] => Command::Shot(Target::Primary),
        ["shot", "--screen", id] => Command::Shot(Target::Screen(num(id)?)),
        ["shot", "--window", id] => Command::Shot(Target::Window(num(id)?)),
        ["shot", "--title", text] => Command::Shot(Target::Title((*text).to_string())),
        ["delete", nr] => Command::Delete(nr.parse().map_err(|_| format!("bad number {nr:?}"))?),
        ["zip", file] => Command::Zip(PathBuf::from(file)),
        _ => return Err(format!("unknown or incomplete command: {}", rest.join(" "))),
    };
    Ok(Invocation { command, dir })
}

fn num(s: &str) -> Result<u64, String> {
    s.parse().map_err(|_| format!("bad id {s:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(a: &[&str]) -> Result<Invocation, String> {
        parse(&a.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn parses_commands_and_dir() {
        let i = p(&["shot", "--screen", "3", "--dir", "x"]).unwrap();
        assert_eq!(i.command, Command::Shot(Target::Screen(3)));
        assert_eq!(i.dir, PathBuf::from("x"));
        assert_eq!(p(&[]).unwrap().command, Command::Help);
        assert_eq!(p(&["delete", "2"]).unwrap().command, Command::Delete(2));
        assert_eq!(p(&["--dir", "d", "list"]).unwrap().dir, PathBuf::from("d"));
    }

    #[test]
    fn rejects_bad_input() {
        assert!(p(&["shot"]).is_err());
        assert!(p(&["shot", "--screen", "x"]).is_err());
        assert!(p(&["delete", "-1"]).is_err());
        assert!(p(&["list", "--dir"]).is_err());
        assert!(p(&["frobnicate"]).is_err());
    }
}
