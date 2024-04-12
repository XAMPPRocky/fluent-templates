use std::fmt;

use unic_langid::LanguageIdentifier;

/// Errors that can occur when loading or parsing fluent resources.
#[derive(Debug, thiserror::Error)]
pub enum LoaderError {
    /// An `io::Error` occurred while interacting with `path`.
    #[error("Error with {}\n: {}", path.display(), source)]
    Fs {
        /// The path to file with the error.
        path: std::path::PathBuf,
        /// The error source.
        source: std::io::Error,
    },
    /// An error was found in the fluent syntax.
    #[error("Error parsing Fluent\n: {}", source)]
    Fluent {
        /// The original parse errors
        #[from]
        source: FluentError,
    },
    /// An error was found whilst loading a bundle at runtime.
    #[error("Failed to add FTL resources to the bundle")]
    FluentBundle {
        /// The original bundle errors
        errors: Vec<fluent_bundle::FluentError>,
    },
}

/// A wrapper struct around `Vec<fluent_syntax::parser::ParserError>`.
#[derive(Debug)]
pub struct FluentError(Vec<fluent_syntax::parser::ParserError>);

impl From<Vec<fluent_syntax::parser::ParserError>> for FluentError {
    fn from(errors: Vec<fluent_syntax::parser::ParserError>) -> Self {
        Self(errors)
    }
}

impl From<FluentError> for Vec<fluent_syntax::parser::ParserError> {
    fn from(error: FluentError) -> Self {
        error.0
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

/// An error that happened while looking up messages
#[derive(Debug, thiserror::Error)]
pub enum LookupError {
    #[error("Couldn't retrieve message with ID `{0}`")]
    MessageRetrieval(String),
    #[error("Couldn't find attribute `{attribute}` for message-id `{message_id}`")]
    AttributeNotFound {
        message_id: String,
        attribute: String,
    },
    #[error("Language ID `{0}` has not been loaded")]
    LangNotLoaded(LanguageIdentifier),
    #[error("Fluent errors: {0:?}")]
    FluentError(Vec<fluent_bundle::FluentError>),
}
