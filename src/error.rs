use std::fmt;

#[derive(Debug, snafu::Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum LoaderError {
    Fs {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    Fluent {
        #[snafu(source(from(Vec<fluent_syntax::parser::ParserError>, FluentError::from)))]
        source: FluentError,
    },
}

#[derive(Debug)]
pub struct FluentError(Vec<fluent_syntax::parser::ParserError>);

impl From<Vec<fluent_syntax::parser::ParserError>> for FluentError {
    fn from(errors: Vec<fluent_syntax::parser::ParserError>) -> Self {
        Self(errors)
    }
}

impl fmt::Display for FluentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for error in &self.0 {
            write!(f, "{:?}", error)?;
        }

        Ok(())
    }
}

impl std::error::Error for FluentError {}
