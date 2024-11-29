use std::{collections::HashMap, hash::Hash};

use crate::{civilization::Civilization, errors::ToolproofInputError, options::ToolproofContext};

use async_trait::async_trait;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum ToolproofSegment {
    Literal(String),
    Value(serde_json::Value),
    Variable(String),
}

#[derive(Debug, Clone)]
pub struct ToolproofSegments {
    pub segments: Vec<ToolproofSegment>,
}

impl Hash for ToolproofSegments {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use ToolproofSegment::*;

        for seg in &self.segments {
            match seg {
                Literal(lit) => lit.hash(state),
                Value(_) | Variable(_) => 0.hash(state),
            }
        }
    }
}

impl PartialEq for ToolproofSegments {
    fn eq(&self, other: &Self) -> bool {
        use ToolproofSegment::*;

        if self.segments.len() != other.segments.len() {
            return false;
        }

        self.segments
            .iter()
            .zip(other.segments.iter())
            .all(|(a, b)| match a {
                Literal(_) => a == b,
                Value(_) | Variable(_) => matches!(b, Variable(_)),
            })
    }
}

impl Eq for ToolproofSegments {}

impl ToolproofSegments {
    pub fn get_variable_names(&self) -> Vec<String> {
        self.segments
            .iter()
            .filter_map(|s| match s {
                ToolproofSegment::Variable(name) => Some(name.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn get_comparison_string(&self) -> String {
        use ToolproofSegment::*;

        self.segments
            .iter()
            .map(|s| match s {
                Literal(l) => l,
                Value(_) | Variable(_) => "{___}",
            })
            .collect()
    }

    pub fn get_as_string(&self) -> String {
        use ToolproofSegment::*;

        self.segments
            .iter()
            .map(|s| match s {
                Literal(l) => l.clone(),
                Value(val) => format!("\"{val}\""),
                Variable(var) => format!("{{{var}}}"),
            })
            .collect()
    }
}

fn has_args_string<V>(args: &HashMap<String, V>) -> String {
    if args.is_empty() {
        "no arguments".to_string()
    } else {
        args.keys().cloned().collect::<Vec<_>>().join(", ")
    }
}

pub struct SegmentArgs<'a> {
    args: HashMap<String, &'a serde_json::Value>,
    placeholder_delim: String,
    placeholders: HashMap<String, String>,
}

impl<'a> SegmentArgs<'a> {
    pub fn build(
        reference_instruction: &ToolproofSegments,
        supplied_instruction: &'a ToolproofSegments,
        supplied_args: &'a HashMap<String, serde_json::Value>,
        civ: Option<&Civilization>,
        transient_placeholders: Option<&HashMap<String, String>>,
    ) -> Result<SegmentArgs<'a>, ToolproofInputError> {
        let mut args = HashMap::new();

        for (reference, supplied) in reference_instruction
            .segments
            .iter()
            .zip(supplied_instruction.segments.iter())
        {
            let ToolproofSegment::Variable(inst_key) = reference else {
                continue;
            };

            match supplied {
                ToolproofSegment::Value(val) => {
                    args.insert(inst_key.to_owned(), val);
                }
                ToolproofSegment::Variable(var) => {
                    let Some(var_val) = supplied_args.get(var) else {
                        return Err(ToolproofInputError::NonexistentArgument {
                            arg: var.to_string(),
                            has: has_args_string(supplied_args),
                        });
                    };
                    args.insert(inst_key.to_owned(), var_val);
                }
                ToolproofSegment::Literal(l) => panic!("{l} should be unreachable"),
            }
        }

        let mut placeholders = civ
            .map(|c| c.universe.ctx.params.placeholders.clone())
            .unwrap_or_default();

        if let Some(civ) = civ {
            placeholders.insert(
                "toolproof_process_directory".to_string(),
                civ.universe
                    .ctx
                    .working_directory
                    .to_string_lossy()
                    .into_owned(),
            );

            if let Some(tmp_dir) = &civ.tmp_dir {
                placeholders.insert(
                    "toolproof_test_directory".to_string(),
                    tmp_dir.path().to_string_lossy().into_owned(),
                );
            }
        }

        if let Some(transient_placeholders) = transient_placeholders {
            placeholders.extend(transient_placeholders.clone().into_iter());
        }

        Ok(Self {
            args,
            placeholders,
            placeholder_delim: civ
                .map(|c| c.universe.ctx.params.placeholder_delimiter.clone())
                .unwrap_or_default(),
        })
    }

    pub fn get_value(&self, k: impl AsRef<str>) -> Result<serde_json::Value, ToolproofInputError> {
        let Some(value) = self.args.get(k.as_ref()) else {
            return Err(ToolproofInputError::NonexistentArgument {
                arg: k.as_ref().to_string(),
                has: has_args_string(&self.args),
            });
        };

        let mut value = (*value).clone();
        replace_inside_value(&mut value, &self.placeholder_delim, &self.placeholders);

        Ok(value)
    }

    pub fn get_string(&self, k: impl AsRef<str>) -> Result<String, ToolproofInputError> {
        let Some(value) = self.args.get(k.as_ref()) else {
            return Err(ToolproofInputError::NonexistentArgument {
                arg: k.as_ref().to_string(),
                has: has_args_string(&self.args),
            });
        };

        let mut value = (*value).clone();
        replace_inside_value(&mut value, &self.placeholder_delim, &self.placeholders);

        let found = match value {
            serde_json::Value::Null => "null",
            serde_json::Value::Bool(_) => "boolean",
            serde_json::Value::Number(_) => "number",
            serde_json::Value::Array(_) => "array",
            serde_json::Value::Object(_) => "object",
            Value::String(st) => return Ok(st),
        };

        return Err(ToolproofInputError::IncorrectArgumentType {
            arg: k.as_ref().to_string(),
            was: found.to_string(),
            expected: "string".to_string(),
        });
    }

    /// Process an arbitrary string as if it were one of the contained arguments
    pub fn process_external_string(&self, raw_value: impl AsRef<str>) -> String {
        let mut value = Value::String(raw_value.as_ref().to_string());
        replace_inside_value(&mut value, &self.placeholder_delim, &self.placeholders);
        match value {
            Value::String(st) => st,
            _ => unreachable!(),
        }
    }
}

fn replace_inside_value(value: &mut Value, delim: &str, placeholders: &HashMap<String, String>) {
    use Value::*;

    match value {
        Null | Bool(_) | Number(_) => {}
        Value::String(s) => {
            if s.contains(delim) {
                for (placeholder, value) in placeholders.iter() {
                    let matcher = format!("{delim}{placeholder}{delim}");

                    if s.contains(&matcher) {
                        *s = s.replace(&matcher, value);
                    }
                }
            }
        }
        Value::Array(vals) => {
            vals.iter_mut().for_each(|v| {
                replace_inside_value(v, delim, placeholders);
            });
        }
        Value::Object(o) => {
            o.values_mut().for_each(|v| {
                replace_inside_value(v, delim, placeholders);
            });
        }
    }
}

#[cfg(test)]
mod test {

    use std::{collections::BTreeMap, sync::Arc};

    use tokio::sync::OnceCell;

    use crate::{
        civilization::Civilization,
        definitions::{register_instructions, ToolproofInstruction},
        errors::ToolproofStepError,
        options::ToolproofParams,
        parser::parse_segments,
        universe::Universe,
    };

    use super::*;

    #[test]
    fn test_building_args() {
        let segments_def = parse_segments("I have a {name} file with the contents {var}")
            .expect("Valid instruction");

        let user_instruction =
            parse_segments("I have a \"index.html\" file with the contents ':)'")
                .expect("Valid instruction");

        let input = HashMap::new();

        let args = SegmentArgs::build(&segments_def, &user_instruction, &input, None, None)
            .expect("Args built successfully");

        let Ok(str) = args.get_string("name") else {
            panic!(
                "Argument was not a string, got {:?}",
                args.get_string("name")
            );
        };
        assert_eq!(str, "index.html");
    }

    #[test]
    fn test_arg_placeholders() {
        let instruction_def = parse_segments("I have a {name} file with the contents {var}")
            .expect("Valid instruction");

        let user_instruction =
            parse_segments("I have a \"%prefix%index.%ext%\" file with the contents ':)'")
                .expect("Valid instruction");

        let input = HashMap::new();
        let mut params = ToolproofParams::default();
        params.placeholders.insert("ext".into(), "pdf".into());
        let ctx = ToolproofContext {
            version: "test",
            working_directory: std::env::current_dir().unwrap(),
            params,
        };

        let universe = Universe {
            browser: OnceCell::new(),
            tests: BTreeMap::new(),
            macros: HashMap::new(),
            macro_comparisons: vec![],
            instructions: HashMap::new(),
            instruction_comparisons: vec![],
            retrievers: HashMap::new(),
            retriever_comparisons: vec![],
            assertions: HashMap::new(),
            assertion_comparisons: vec![],
            ctx,
        };

        let civ = Civilization {
            tmp_dir: None,
            last_command_output: None,
            assigned_server_port: None,
            window: None,
            threads: vec![],
            handles: vec![],
            env_vars: HashMap::new(),
            universe: Arc::new(universe),
        };

        let args = SegmentArgs::build(
            &instruction_def,
            &user_instruction,
            &input,
            Some(&civ),
            Some(&HashMap::from([("prefix".to_string(), "__".to_string())])),
        )
        .expect("Args built successfully");

        let Ok(str) = args.get_string("name") else {
            panic!(
                "Argument was not a string, got {:?}",
                args.get_string("name")
            );
        };
        assert_eq!(str, "__index.pdf");
    }

    // Segments should alias to each other regardless of the contents of their
    // variables or values.
    #[test]
    fn test_segments_equality() {
        let segments_a = parse_segments("I have a 'index.html' file with the contents {var}")
            .expect("Valid segments");

        let segments_b = parse_segments("I have a {filename} file with the contents {var}")
            .expect("Valid segments");

        let segments_c = parse_segments("I have one {filename} file with the contents {var}")
            .expect("Valid segments");

        assert_eq!(segments_a, segments_b);

        let mut map = HashMap::new();
        map.insert(&segments_b, "b");

        assert_eq!(map.get(&&segments_a), Some(&"b"));

        assert_ne!(segments_b, segments_c);
        assert_eq!(map.get(&&segments_c), None);
    }

    #[test]
    fn test_complex_placeholders() {
        let placeholders = HashMap::from([
            ("cloud".to_string(), "cannon".to_string()),
            ("thekey".to_string(), "the value".to_string()),
        ]);

        let start_value: serde_json::Value = serde_json::from_str(
            r#"
            {
                "title": "Hello cloud%cloud%",
                "tags": [ "cannon", "%cloud%" ],
                "nested": {
                    "null": null,
                    "count": 3,
                    "replaced": "thekey is %thekey%"
                }
            }
        "#,
        )
        .unwrap();

        let mut end_value = start_value.clone();
        replace_inside_value(&mut end_value, "%", &placeholders);

        let expected_end_value: serde_json::Value = serde_json::from_str(
            r#"
            {
                "title": "Hello cloudcannon",
                "tags": [ "cannon", "cannon" ],
                "nested": {
                    "null": null,
                    "count": 3,
                    "replaced": "thekey is the value"
                }
            }
        "#,
        )
        .unwrap();

        assert_eq!(end_value, expected_end_value);
    }
}
