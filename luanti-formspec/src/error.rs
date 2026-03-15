use std::{fmt::Display, num::ParseIntError};

#[derive(Debug)]
pub(crate) enum FormspecError {
    NameIsQuit,
    NameIsKey,
    NameIsEmpty,
    NameInvalidChar(String),
    PrematureEnd,
    UnknownElement(String),
    ExcessiveArguments(&'static str, String),
    Argument {
        element_name: &'static str,
        argument_name: &'static str,
        error: ArgumentError,
    },
}

impl FormspecError {
    pub(crate) fn argument(
        element_name: &'static str,
        argument_name: &'static str,
        error: ArgumentError,
    ) -> Self {
        Self::Argument {
            element_name,
            argument_name,
            error,
        }
    }
}

#[derive(Debug)]
pub(crate) enum ArgumentError {
    Missing,
    ParseIntError(ParseIntError),
    Excessive(String),
}

impl Display for FormspecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NameIsQuit => f.write_str("'quit' is a reserved formspec element name"),
            Self::NameIsKey => f.write_str(
                "formspec element names starting with 'key_' are reserved to pass key press events",
            ),
            Self::NameIsEmpty => f.write_str("formspec element names may not be empty"),
            Self::NameInvalidChar(name) => write!(
                f,
                "formspec element name contains at least one illegal character: {name}"
            ),
            Self::PrematureEnd => f.write_str("premature end of formspec string"),
            Self::UnknownElement(name) => write!(f, "unknown formspec element: '{name}'"),
            Self::Argument {
                element_name,
                argument_name,
                error,
            } => {
                write!(f, "{error} in {element_name}:{argument_name}")
            }
            Self::ExcessiveArguments(element_name, args) => {
                write!(
                    f,
                    "excessive arguments for formspec element '{element_name}': args"
                )
            }
        }
    }
}

impl Display for ArgumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgumentError::Missing => f.write_str("missing argument"),
            ArgumentError::ParseIntError(parse_int_error) => {
                write!(f, "invalid integer ({parse_int_error})")
            }
            ArgumentError::Excessive(excessive) => {
                write!(f, "excessive components ({excessive})")
            }
        }
    }
}
