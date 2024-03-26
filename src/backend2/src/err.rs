use std::borrow::Cow;
use std::convert::Infallible;
use std::path::Path;

use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use lc3_ensemble::err::ErrSpan;
use neon::context::Context;
use neon::result::Throw;

pub(crate) fn simple_reporter<E: std::fmt::Display + ?Sized>(err: &E) -> Reporter<'_, E> {
    Reporter {
        err,
        filename: None,
        source: None,
        span: None,
        help: None,
        msg_includes_fname: true
    }
}

pub(crate) fn io_reporter<'a, E: std::fmt::Display + ?Sized>(err: &'a E, fp: &'a Path) -> Reporter<'a, E> {
    Reporter {
        err,
        filename: fp.file_name().and_then(|s| s.to_str()),
        source: None,
        span: None,
        help: None,
        msg_includes_fname: true
    }
}
pub(crate) fn error_reporter<'a, E: lc3_ensemble::err::Error + ?Sized>(err: &'a E, fp: &'a Path, src: &'a str) -> Reporter<'a, E> {
    let span = err.span();
    let help = err.help();

    Reporter {
        err,
        filename: fp.file_name().and_then(|s| s.to_str()),
        source: Some(Source::from(src.to_string())),
        span,
        help,
        msg_includes_fname: false,
    }
}

pub(crate) struct Reporter<'c, E: ?Sized> {
    err: &'c E,
    filename: Option<&'c str>,
    source: Option<Source<String>>,
    span: Option<ErrSpan>,
    help: Option<Cow<'c, str>>,
    msg_includes_fname: bool
}

impl<E: std::fmt::Display + ?Sized> Reporter<'_, E> {
    pub(crate) fn report(&mut self, writer: &mut impl std::io::Write) {
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
        match self.span.clone() {
            Some(ErrSpan::One(r)) => {
                report.add_label({
                    let mut label = Label::new((fname, r))
                        .with_color(colors.next());
                    
                    if let Some(help) = self.help.as_deref() {
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

                if let Some(help) = self.help.as_deref() {
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

                if let Some(help) = self.help.as_deref() {
                    report.set_help(help);
                }
            },
            None => {
                if let Some(help) = self.help.as_deref() {
                    report.add_label(Label::new((fname, 0..0)));
                    report.set_help(help);
                };
            }
        }
        
        let source = match &self.source {
            Some(s) => s.clone(),
            None    => Source::from(String::new()),
        };
        report.finish()
            .write((fname, source), writer)
            .unwrap();
    }

    pub(crate) fn report_and_throw<'a>(mut self, writer: &mut impl std::io::Write, cx: &mut impl Context<'a>) -> Throw {
        self.report(writer);

        cx.throw_error::<_, Infallible>(self.err.to_string())
            .unwrap_err()
    }
}