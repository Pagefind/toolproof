use std::{collections::HashMap, path::PathBuf};

use path_slash::{PathBufExt, PathExt};
use serde_json::{Map, Value};

use crate::{
    errors::ToolproofInputError,
    platforms::normalize_line_endings,
    segments::{ToolproofSegment, ToolproofSegments},
    ToolproofTestFile, ToolproofTestStep, ToolproofTestStepState,
};

struct ToolproofTestInput {
    parsed: RawToolproofTestFile,
    original_source: String,
    file_path: String,
    file_directory: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolproofFileType {
    Test,
    Reference,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct RawToolproofTestFile {
    name: String,
    r#type: Option<ToolproofFileType>,
    steps: Vec<RawToolproofTestStep>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
enum RawToolproofTestStep {
    Ref {
        r#ref: String,
    },
    BareStep(String),
    StepWithParams {
        step: String,
        #[serde(flatten)]
        other: Map<String, Value>,
    },
    Snapshot {
        snapshot: String,
        #[serde(flatten)]
        other: Map<String, Value>,
    },
}

impl TryFrom<ToolproofTestInput> for ToolproofTestFile {
    type Error = ToolproofInputError;

    fn try_from(value: ToolproofTestInput) -> Result<Self, Self::Error> {
        let mut steps = Vec::with_capacity(value.parsed.steps.len());
        for step in value.parsed.steps {
            steps.push(step.try_into()?);
        }

        Ok(ToolproofTestFile {
            name: value.parsed.name,
            r#type: value.parsed.r#type.unwrap_or(ToolproofFileType::Test),
            steps,
            original_source: value.original_source,
            file_path: value.file_path,
            file_directory: value.file_directory,
        })
    }
}

impl TryFrom<RawToolproofTestStep> for ToolproofTestStep {
    type Error = ToolproofInputError;

    fn try_from(value: RawToolproofTestStep) -> Result<Self, Self::Error> {
        match value {
            RawToolproofTestStep::Ref { r#ref } => Ok(ToolproofTestStep::Ref {
                other_file: PathBuf::try_from(&r#ref)
                    .map_err(|_| ToolproofInputError::InvalidPath {
                        input: r#ref.clone(),
                    })?
                    .to_slash_lossy()
                    .into_owned(),
                orig: r#ref,
                hydrated_steps: None,
                state: ToolproofTestStepState::Dormant,
            }),
            RawToolproofTestStep::BareStep(step) => parse_step(step, HashMap::new()),
            RawToolproofTestStep::StepWithParams { step, other } => {
                parse_step(step, HashMap::from_iter(other.into_iter()))
            }
            RawToolproofTestStep::Snapshot { snapshot, other } => Ok(ToolproofTestStep::Snapshot {
                snapshot: parse_segments(&snapshot)?,
                snapshot_content: None,
                args: HashMap::from_iter(other.into_iter()),
                orig: snapshot,
                state: ToolproofTestStepState::Dormant,
            }),
        }
    }
}

fn parse_step(
    step: String,
    args: HashMap<String, Value>,
) -> Result<ToolproofTestStep, ToolproofInputError> {
    if let Some((retrieval, assertion)) = step.split_once(" should ") {
        Ok(ToolproofTestStep::Assertion {
            retrieval: parse_segments(retrieval)?,
            assertion: parse_segments(assertion)?,
            args,
            orig: step,
            state: ToolproofTestStepState::Dormant,
        })
    } else {
        Ok(ToolproofTestStep::Instruction {
            step: parse_segments(&step)?,
            args,
            orig: step,
            state: ToolproofTestStepState::Dormant,
        })
    }
}

pub fn parse_file(s: &str, p: PathBuf) -> Result<ToolproofTestFile, ToolproofInputError> {
    let raw_test = serde_yaml::from_str::<RawToolproofTestFile>(s)?;

    ToolproofTestInput {
        parsed: raw_test,
        original_source: normalize_line_endings(s),
        file_path: p.to_slash_lossy().into_owned(),
        file_directory: p
            .parent()
            .map(|p| p.to_slash_lossy().into_owned())
            .unwrap_or_else(|| ".".to_string()),
    }
    .try_into()
}

pub fn parse_segments(s: &str) -> Result<ToolproofSegments, ToolproofInputError> {
    let mut segments = vec![];
    use ToolproofSegment::*;

    enum InstMode {
        None(usize),
        InQuote(usize, char),
        InCurly(usize),
    }

    let mut mode = InstMode::None(0);

    for (i, c) in s.char_indices() {
        match &mut mode {
            InstMode::None(start) => match c {
                '"' => {
                    segments.push(Literal(s[*start..i].to_lowercase()));
                    mode = InstMode::InQuote(i, '"');
                }
                '\'' => {
                    segments.push(Literal(s[*start..i].to_lowercase()));
                    mode = InstMode::InQuote(i, '\'');
                }
                '{' => {
                    segments.push(Literal(s[*start..i].to_lowercase()));
                    mode = InstMode::InCurly(i);
                }
                _ => {}
            },
            InstMode::InQuote(start, quote) => match c {
                c if c == *quote => {
                    let inner_start = *start + 1;
                    if i == inner_start {
                        segments.push(Value(serde_json::Value::String("".to_string())));
                    } else {
                        segments.push(Value(serde_json::Value::String(
                            s[inner_start..i].to_string(),
                        )));
                    }
                    mode = InstMode::None(i + 1);
                }
                _ => {}
            },
            InstMode::InCurly(start) => match c {
                '}' => {
                    let inner_start = *start + 1;
                    if i == inner_start {
                        segments.push(Variable("".to_string()));
                    } else {
                        segments.push(Variable(s[inner_start..i].to_string()));
                    }
                    mode = InstMode::None(i + 1);
                }
                _ => {}
            },
        }
    }

    match mode {
        InstMode::None(start) => {
            if start < s.len() {
                segments.push(Literal(s[start..].to_lowercase()));
            }
        }
        InstMode::InQuote(_, q) => return Err(ToolproofInputError::UnclosedValue { expected: q }),
        InstMode::InCurly(_) => return Err(ToolproofInputError::UnclosedValue { expected: '}' }),
    }

    Ok(ToolproofSegments { segments })
}

#[cfg(test)]
mod test {
    use super::*;
    use ToolproofSegment::*;

    fn st(s: &str) -> serde_json::Value {
        serde_json::Value::String(s.to_string())
    }

    #[test]
    fn test_parsing_segments() {
        let segments = parse_segments("I run my program").expect("Valid segments");
        // We test equality on the segments directly,
        // as the segments itself uses a looser comparison that doesn't
        // look inside Value or Variable segments.
        assert_eq!(
            segments.segments,
            vec![Literal("i run my program".to_string())]
        );

        let segments = parse_segments("I have a \"public/cat/'index'.html\" file with the body '<h1>Happy post about \"cats</h1>'").expect("Valid segments");
        assert_eq!(
            segments.segments,
            vec![
                Literal("i have a ".to_string()),
                Value(st("public/cat/'index'.html")),
                Literal(" file with the body ".to_string()),
                Value(st("<h1>Happy post about \"cats</h1>"))
            ]
        );

        let segments =
            parse_segments("In my browser, ''I eval {j\"s} and 'x'").expect("Valid segments");
        assert_eq!(
            segments.segments,
            vec![
                Literal("in my browser, ".to_string()),
                Value(st("")),
                Literal("i eval ".to_string()),
                Variable("j\"s".to_string()),
                Literal(" and ".to_string()),
                Value(st("x")),
            ]
        );
    }

    #[test]
    fn test_parsing_steps() {
        let Ok(step) = parse_step("I have a {js} file".to_string(), HashMap::new()) else {
            panic!("Step did not parse");
        };

        assert_eq!(
            step,
            ToolproofTestStep::Instruction {
                step: ToolproofSegments {
                    segments: vec![
                        Literal("i have a ".to_string()),
                        Variable("js".to_string()),
                        Literal(" file".to_string())
                    ]
                },
                args: HashMap::new(),
                orig: "I have a {js} file".to_string(),
                state: ToolproofTestStepState::Dormant
            }
        );

        let Ok(step) = parse_step(
            "The file {name} should contain {html}".to_string(),
            HashMap::new(),
        ) else {
            panic!("Step did not parse");
        };

        assert_eq!(
            step,
            ToolproofTestStep::Assertion {
                retrieval: ToolproofSegments {
                    segments: vec![
                        Literal("the file ".to_string()),
                        Variable("name".to_string())
                    ]
                },
                assertion: ToolproofSegments {
                    segments: vec![
                        Literal("contain ".to_string()),
                        Variable("html".to_string()),
                    ]
                },
                args: HashMap::new(),
                orig: "The file {name} should contain {html}".to_string(),
                state: ToolproofTestStepState::Dormant
            }
        );
    }
}
