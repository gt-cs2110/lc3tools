use std::convert::Infallible;
use std::path::Path;

use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use lc3_ensemble::err::ErrSpan;
use neon::context::Context;
use neon::result::Throw;

pub(crate) struct InputBuffer {
    tx: crossbeam_channel::Sender<u8>,
    rx: crossbeam_channel::Receiver<u8>
}
impl InputBuffer {
    pub(crate) fn new() -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        InputBuffer { tx, rx }
    }

    pub(crate) fn send(&self, byte: u8) {
        // shouldn't ever disconnect
        let _ = self.tx.send(byte);
    }

    /// Retrieves a new receiver channel.
    /// 
    /// This is done to prevent the receiver from being used in blocking scenarios
    /// and obstructing a lock.
    pub(crate) fn rx(&self) -> crossbeam_channel::Receiver<u8> {
        self.rx.clone()
    }
}
#[derive(Default)]
pub(crate) struct PrintBuffer(String);
impl PrintBuffer {
    pub(crate) const fn new() -> Self {
        PrintBuffer(String::new())
    }
    pub(crate) fn take(&mut self) -> String {
        std::mem::take(&mut self.0)
    }
}
impl std::io::Write for PrintBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        use std::io;

        let string = std::str::from_utf8(buf)
            .map_err(io::Error::other)?;
        self.0.push_str(string);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(crate) fn report_simple<'a>(fp: &Path, err: impl std::fmt::Display, cx: &mut impl Context<'a>, writer: &mut PrintBuffer) -> Throw {
    ReportContents {
        writer,
        err: &err,
        filename: fp.file_name().and_then(|s| s.to_str()),
        source: None,
        span: None,
        help: None,
        msg_includes_fname: true
    }.report(cx)
}
pub(crate) fn report_error<'a, E: lc3_ensemble::err::Error>(err: E, fp: &Path, src: &str, cx: &mut impl Context<'a>, writer: &mut PrintBuffer) -> Throw {
    let span = err.span();
    let help = err.help();

    ReportContents {
        writer,
        err: &err,
        filename: fp.file_name().and_then(|s| s.to_str()),
        source: Some(Source::from(src.to_string())),
        span,
        help: help.as_deref(),
        msg_includes_fname: false,
    }.report(cx)
}

struct ReportContents<'c, 'w, E> {
    writer: &'w mut PrintBuffer,
    err: &'c E,
    filename: Option<&'c str>,
    source: Option<Source<String>>,
    span: Option<ErrSpan>,
    help: Option<&'c str>,
    msg_includes_fname: bool
}

impl<E: std::fmt::Display> ReportContents<'_, '_, E> {
    fn report<'a>(self, cx: &mut impl Context<'a>) -> Throw {
        let mut colors = ColorGenerator::new();

        let msg = if self.msg_includes_fname {
            if let Some(fname) = self.filename {
                format!("{}: {}", fname, self.err)
            } else {
                self.err.to_string()
            }
        } else {
            self.err.to_string()
        };
        let fname = self.filename.unwrap_or("source");
        let offset = self.span.as_ref().map_or(0, |e| e.first().start);
        
        let mut report = Report::build(ReportKind::Error, fname, offset).with_message(msg);
        match self.span {
            Some(ErrSpan::One(r)) => {
                report.add_label({
                    let mut label = Label::new((fname, r))
                        .with_color(colors.next());
                    
                    if let Some(help) = self.help {
                        label = label.with_message(help);
                    }

                    label
                })
            },
            Some(ErrSpan::Two([r0, r1])) => {
                report.add_label({
                    Label::new((fname, r0))
                            .with_color(colors.next())
                            .with_message("")
                });
                report.add_label({
                    Label::new((fname, r1))
                            .with_color(colors.next())
                            .with_message("")
                });

                if let Some(help) = self.help {
                    report.set_help(help);
                }
            },
            Some(ErrSpan::Many(mr)) => {
                report.add_labels({
                    mr.into_iter()
                        .map(|s| {
                            Label::new((fname, s.clone()))
                                .with_color(colors.next())
                                .with_message("")
                        })
                });

                if let Some(help) = self.help {
                    report.set_help(help);
                }
            },
            None => {
                if let Some(help) = self.help {
                    report.add_label(Label::new((fname, 0..0)));
                    report.set_help(help);
                };
            }
        }
        
        report.finish()
            .write((fname, self.source.unwrap_or_else(|| Source::from(String::new()))), self.writer)
            .unwrap();

        cx.throw_error::<_, Infallible>(self.err.to_string())
            .unwrap_err()
    }
}