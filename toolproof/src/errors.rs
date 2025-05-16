use std::path::PathBuf;

use chromiumoxide::error::CdpError;
use pagebrowse::PagebrowseError;
use thiserror::Error;

use crate::ToolproofTestStep;

#[derive(Error, Debug)]
pub enum ToolproofInputError {
    #[error("Argument \"{arg}\" was not supplied. (have {has})")]
    NonexistentArgument { arg: String, has: String },
    #[error("Argument \"{arg}\" expected to be a {expected}, but is a {was}")]
    IncorrectArgumentType {
        arg: String,
        was: String,
        expected: String,
    },
    #[error("Argument \"{arg}\" requires a value, cannot be empty")]
    ArgumentRequiresValue { arg: String },
    #[error("File {filename} failed to parse: {inner}")]
    ParseError {
        filename: String,
        inner: serde_yaml::Error,
    },
    #[error("unclosed argument, expected a {expected} character")]
    UnclosedValue { expected: char },
    #[error("invalid path: \"{input}\"")]
    InvalidPath { input: String },
    #[error("duplicate name of \"{name}\" on the files {path_one} and {path_two}")]
    DuplicateName {
        path_one: String,
        path_two: String,
        name: String,
    },
    #[error("invalid reference: \"{input}\". (closest available: \"{closest}\")")]
    InvalidRef { input: String, closest: String },
    #[error("step does not exist")]
    NonexistentStep,
    #[error("step requirements were not met: {reason}")]
    StepRequirementsNotMet { reason: String },
    #[error("{reason}")]
    StepError { reason: String },
}

#[derive(Error, Debug)]
pub enum ToolproofInternalError {
    #[error("Test error: {msg}")]
    Custom { msg: String },
    #[error("{0}")]
    PagebrowseError(#[from] PagebrowseError),
    #[error("{0}")]
    ChromeError(#[from] CdpError),
}

#[derive(Error, Debug)]
pub enum ToolproofTestFailure {
    #[error("{msg}")]
    Custom { msg: String },
    #[error("{msg}\nbrowser console:\n{logs}")]
    BrowserJavascriptErr { msg: String, logs: String },
}

#[derive(Error, Debug)]
pub enum ToolproofStepError {
    #[error("Parse error: {0}")]
    External(#[from] ToolproofInputError),
    #[error("Step error: {0}")]
    Internal(#[from] ToolproofInternalError),
    #[error("Failed assertion: {0}")]
    Assertion(#[from] ToolproofTestFailure),
}

#[derive(Error, Debug)]
#[error("Error in step \"{step}\":\n{arg_str}--\n{err}")]
pub struct ToolproofTestError {
    pub err: ToolproofStepError,
    pub step: ToolproofTestStep,
    pub arg_str: String,
}
