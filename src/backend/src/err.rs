use std::borrow::Cow;
use std::convert::Infallible;
use std::path::Path;
use std::sync::LazyLock;

use lc3_ensemble::err::ErrSpan;
use miette::highlighters::{Highlighter, HighlighterState};
use miette::{Diagnostic, GraphicalReportHandler, GraphicalTheme, NamedSource, Severity, ThemeCharacters, ThemeStyles};
use neon::context::Context;
use neon::result::Throw;
use owo_colors::style;

struct FlatHighlighter(owo_colors::Style);
impl Highlighter for FlatHighlighter {
    fn start_highlighter_state<'h>(
        &'h self,
        _: &dyn miette::SpanContents<'_>,
    ) -> Box<dyn HighlighterState + 'h> {
        struct Highlighted(owo_colors::Style);
        impl HighlighterState for Highlighted {
            fn highlight_line<'s>(&mut self, line: &'s str) -> Vec<owo_colors::Styled<&'s str>> {
                vec![self.0.style(line)]
            }
        }

        Box::new(Highlighted(self.0))
    }
}
static REPORT_HANDLER: LazyLock<GraphicalReportHandler> = LazyLock::new(|| {
    let style_dimmed = style().fg_rgb::<0xA0, 0xA0, 0xA0>();

    GraphicalReportHandler::new_themed(GraphicalTheme {
        characters: ThemeCharacters {
            error: String::from("Error:"),
            warning: String::from("Warning:"),
            advice: String::from("Info:"),
            ..ThemeCharacters::unicode()
        },
        styles: ThemeStyles {
            link: style(),
            linum: style_dimmed,
            highlights: vec![
                style().magenta(),
                style().yellow(),
                style().green(),
            ],
            ..ThemeStyles::ansi()
        }
    })
    .with_context_lines(1)
    .with_syntax_highlighting(FlatHighlighter(style_dimmed)) // Make code gray
});

#[derive(Debug)]
enum ReporterSource<'s> {
    Unlabeled(&'s str),
    Labeled(NamedSource<String>)
}
impl<'s> ReporterSource<'s> {
    fn new<'a>(filename: Option<&'a str>, source: &'s str) -> Self {
        match filename {
            Some(name) => ReporterSource::Labeled(NamedSource::new(name, source.to_string())),
            None => ReporterSource::Unlabeled(source),
        }
    }
}
impl miette::SourceCode for ReporterSource<'_> {
    fn read_span<'a>(
        &'a self,
        span: &miette::SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
        match self {
            ReporterSource::Unlabeled(s) => s.read_span(span, context_lines_before, context_lines_after),
            ReporterSource::Labeled(s) => s.read_span(span, context_lines_before, context_lines_after),
        }
    }
}

pub(crate) struct Reporter<'r, E: std::fmt::Display + ?Sized> {
    /// Error (and message to print)
    err: &'r E,
    /// The filename of file where this error occurred (if there is a file)
    filename: Option<&'r str>,
    /// Source code
    source: Option<ReporterSource<'r>>,
    /// Span where error occurred
    span: Option<ErrSpan>,
    /// Relevant help messages
    help: Option<Cow<'r, str>>,
    /// Whether to include the filename in the error message
    /// (can be false if it'd already appear elsewhere)
    include_name_in_msg: bool
}
impl<E: std::fmt::Display + ?Sized> std::fmt::Debug for Reporter<'_, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reporter")
            .field("err", &self.err.to_string())
            .field("filename", &self.filename)
            .field("source", &self.source)
            .field("span", &self.span)
            .field("help", &self.help)
            .field("include_name_in_msg", &self.include_name_in_msg)
            .finish()
    }
}
impl<E: std::fmt::Display + ?Sized> std::fmt::Display for Reporter<'_, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.filename {
            Some(filename) if self.include_name_in_msg => write!(f, "{filename}: {}", self.err),
            _ => write!(f, "{}", self.err),
        }
    }
}
impl<E: std::fmt::Display + ?Sized> std::error::Error for Reporter<'_, E> {}
impl<E: std::fmt::Display + ?Sized> Diagnostic for Reporter<'_, E> {
    fn severity(&self) -> Option<Severity> {
        Some(Severity::Error)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.help.as_ref().filter(|s| !s.is_empty()).map(|s| Box::new(s) as _)
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        self.source.as_ref().map(|s| s as &dyn miette::SourceCode)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        self.span.as_ref().map(|s| Box::new(
            s.iter().map(|s| miette::LabeledSpan::new_with_span(None, s.clone()))
        ) as _)
    }
}

impl<'r, E: std::fmt::Display + ?Sized> Reporter<'r, E> {
    pub(crate) fn simple(err: &'r E) -> Self {
        Reporter {
            err,
            filename: None,
            source: None,
            span: None,
            help: None,
            include_name_in_msg: true
        }
    }
    
    pub(crate) fn io(err: &'r E, fp: &'r Path) -> Self {
        Reporter {
            err,
            filename: fp.file_name().and_then(|s| s.to_str()),
            source: None,
            span: None,
            help: None,
            include_name_in_msg: true
        }
    }
    pub(crate) fn ensemble(err: &'r E, fp: &'r Path, src: &'r str) -> Self
        where E: lc3_ensemble::err::Error
    {
        let span = err.span();
        let help = err.help();
        let filename = fp.file_name().and_then(|s| s.to_str());
        Reporter {
            err,
            filename,
            source: Some(ReporterSource::new(filename, src)),
            span,
            help,
            include_name_in_msg: false,
        }
    }
}

impl<E: std::fmt::Display + ?Sized> Reporter<'_, E> {
    pub(crate) fn report(&mut self, writer: &mut Vec<u8>) {
        let mut report = String::new();
        REPORT_HANDLER.render_report(&mut report, self).unwrap();

        // Remove whitespace that miette adds at the beginning of reports
        writer.extend(report.trim_start().as_bytes());
    }

    pub(crate) fn report_and_throw<'a>(mut self, writer: &mut Vec<u8>, cx: &mut impl Context<'a>) -> Throw {
        self.report(writer);

        cx.throw_error::<_, Infallible>(self.err.to_string())
            .unwrap_err()
    }
}