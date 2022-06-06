use termcolor::{StandardStream, ColorChoice, WriteColor, ColorSpec, Color};
use std::io::Write;

pub struct ShellOutput {
    stdout: StandardStream,
    stderr: StandardStream,
}

impl ShellOutput {
    pub fn new() -> Self {
        ShellOutput {
            stdout: StandardStream::stdout(ColorChoice::Auto),
            stderr: StandardStream::stderr(ColorChoice::Auto),
        }
    }

    pub fn message<T: std::fmt::Display>(&self, heading: T, caption: T) {
        let mut so = self.stdout.lock();
        so.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_intense(true).set_bold(true)).unwrap();
        write!(so, "{:>12}", heading).unwrap();
        so.set_color(ColorSpec::new().set_reset(true)).unwrap();
        write!(so, " {}\n", caption).unwrap();
    }

    pub fn message_err<T: std::fmt::Display>(&self, heading: T, caption: T) {
        let mut so = self.stderr.lock();
        so.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_intense(true).set_bold(true)).unwrap();
        write!(so, "{:>12}", heading).unwrap();
        so.set_color(ColorSpec::new().set_reset(true)).unwrap();
        write!(so, " {}\n", caption).unwrap();
    }
}