use std::collections::HashMap;

use async_trait::async_trait;

use crate::civilization::Civilization;
use crate::errors::{ToolproofInputError, ToolproofInternalError, ToolproofStepError};

use super::{SegmentArgs, ToolproofAssertion, ToolproofInstruction, ToolproofRetriever};

fn value_contains_value(
    base: &serde_json::Value,
    expected: &serde_json::Value,
) -> Result<bool, ToolproofStepError> {
    use serde_json::Value::*;

    if base == expected {
        return Ok(true);
    };

    match (&base, &expected) {
        (Null, _) => Ok(false),
        (Bool(_), _) => Ok(false),
        (Number(_), _) => Ok(false),
        (String(s), Bool(b)) => {
            if s.contains(&b.to_string()) {
                Ok(true)
            } else {
                Ok(false)
            }
        },
        (String(s), Number(n)) => {
            if s.contains(&n.to_string()) {
                Ok(true)
            } else {
                Ok(false)
            }
        },
        (String(s), String(s2)) => {
            if s.contains(s2) {
                Ok(true)
            } else {
                Ok(false)
            }
        },
        // (Array(_), Null) => todo!(),
        // (Array(_), Bool(_)) => todo!(),
        // (Array(_), Number(_)) => todo!(),
        // (Array(_), String(_)) => todo!(),
        // (Array(_), Array(_)) => todo!(),
        // (Array(_), Object(_)) => todo!(),
        // (Object(_), Null) => todo!(),
        // (Object(_), Bool(_)) => todo!(),
        // (Object(_), Number(_)) => todo!(),
        // (Object(_), String(_)) => todo!(),
        // (Object(_), Array(_)) => todo!(),
        // (Object(_), Object(_)) => todo!(),
        _ => {
            Err(ToolproofStepError::Internal(ToolproofInternalError::Custom { msg: format!(
                "A comparison for these values has not been implemented.\n---\n{}\n---\ncannot compare with\n---\n{}\n---",
                serde_json::to_string(&base).expect("should be yaml-able"),
                serde_json::to_string(&expected).expect("should be yaml-able")
            ) }))
        }
    }
}

fn value_is_empty(val: &serde_json::Value) -> bool {
    match val {
        serde_json::Value::Null => true,
        serde_json::Value::String(s) => s.is_empty(),
        serde_json::Value::Bool(_) => false,
        serde_json::Value::Number(_) => false,
        serde_json::Value::Array(a) => a.is_empty(),
        serde_json::Value::Object(o) => o.is_empty(),
    }
}

fn value_type(val: &serde_json::Value) -> &'static str {
    match val {
        serde_json::Value::Null => "null",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

mod contain {
    use crate::errors::{ToolproofInternalError, ToolproofTestFailure};

    use super::*;

    pub struct Contain;

    inventory::submit! {
        &Contain as &dyn ToolproofAssertion
    }

    #[async_trait]
    impl ToolproofAssertion for Contain {
        fn segments(&self) -> &'static str {
            "contain {expected}"
        }

        async fn run(
            &self,
            base_value: serde_json::Value,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<(), ToolproofStepError> {
            let expected = args.get_value("expected")?;

            if value_contains_value(&base_value, &expected)? {
                Ok(())
            } else {
                Err(ToolproofStepError::Assertion(
                    ToolproofTestFailure::Custom {
                        msg: format!(
                            "The value\n---\n{}\n---\ndoes not contain\n---\n{}\n---",
                            serde_json::to_string(&base_value).expect("should be yaml-able"),
                            serde_json::to_string(&expected).expect("should be yaml-able")
                        ),
                    },
                ))
            }
        }
    }

    pub struct NotContain;

    inventory::submit! {
        &NotContain as &dyn ToolproofAssertion
    }

    #[async_trait]
    impl ToolproofAssertion for NotContain {
        fn segments(&self) -> &'static str {
            "not contain {expected}"
        }

        async fn run(
            &self,
            base_value: serde_json::Value,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<(), ToolproofStepError> {
            let expected = args.get_value("expected")?;

            if value_contains_value(&base_value, &expected)? {
                Err(ToolproofStepError::Assertion(ToolproofTestFailure::Custom {
                    msg: format!(
                        "The value\n---\n{}\n---\nshould not contain the following value, but does\n---\n{}\n---",
                        serde_json::to_string(&base_value).expect("should be yaml-able"),
                        serde_json::to_string(&expected).expect("should be yaml-able")
                    ),
                }))
            } else {
                Ok(())
            }
        }
    }
}

mod exactly {
    use crate::errors::{ToolproofInternalError, ToolproofTestFailure};

    use super::*;

    pub struct Exactly;

    inventory::submit! {
        &Exactly as &dyn ToolproofAssertion
    }

    #[async_trait]
    impl ToolproofAssertion for Exactly {
        fn segments(&self) -> &'static str {
            "be exactly {expected}"
        }

        async fn run(
            &self,
            base_value: serde_json::Value,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<(), ToolproofStepError> {
            let expected = args.get_value("expected")?;

            if base_value == expected {
                Ok(())
            } else {
                Err(ToolproofStepError::Assertion(ToolproofTestFailure::Custom {
                    msg: format!(
                        "The value\n---\n{}\n---\nshould be exactly the following value, but is not\n---\n{}\n---",
                        serde_json::to_string(&base_value).expect("should be yaml-able"),
                        serde_json::to_string(&expected).expect("should be yaml-able")
                    ),
                }))
            }
        }
    }

    pub struct NotExactly;

    inventory::submit! {
        &NotExactly as &dyn ToolproofAssertion
    }

    #[async_trait]
    impl ToolproofAssertion for NotExactly {
        fn segments(&self) -> &'static str {
            "not be exactly {expected}"
        }

        async fn run(
            &self,
            base_value: serde_json::Value,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<(), ToolproofStepError> {
            let expected = args.get_value("expected")?;

            if base_value != expected {
                Ok(())
            } else {
                Err(ToolproofStepError::Assertion(ToolproofTestFailure::Custom {
                    msg: format!(
                        "The value\n---\n{}\n---\nshould be exactly the following value, but is not\n---\n{}\n---",
                        serde_json::to_string(&base_value).expect("should be yaml-able"),
                        serde_json::to_string(&expected).expect("should be yaml-able")
                    ),
                }))
            }
        }
    }
}

mod empty {
    use crate::errors::{ToolproofInternalError, ToolproofTestFailure};

    use super::*;

    pub struct Empty;

    inventory::submit! {
        &Empty as &dyn ToolproofAssertion
    }

    #[async_trait]
    impl ToolproofAssertion for Empty {
        fn segments(&self) -> &'static str {
            "be empty"
        }

        async fn run(
            &self,
            base_value: serde_json::Value,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<(), ToolproofStepError> {
            if value_is_empty(&base_value) {
                Ok(())
            } else {
                Err(ToolproofStepError::Assertion(
                    ToolproofTestFailure::Custom {
                        msg: format!(
                            "The value should be empty, but was:\n---\n{}\n---",
                            serde_json::to_string(&base_value).expect("should be yaml-able"),
                        ),
                    },
                ))
            }
        }
    }

    pub struct NotEmpty;

    inventory::submit! {
        &NotEmpty as &dyn ToolproofAssertion
    }

    #[async_trait]
    impl ToolproofAssertion for NotEmpty {
        fn segments(&self) -> &'static str {
            "not be empty"
        }

        async fn run(
            &self,
            base_value: serde_json::Value,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<(), ToolproofStepError> {
            if value_is_empty(&base_value) {
                Err(ToolproofStepError::Assertion(
                    ToolproofTestFailure::Custom {
                        msg: format!(
                            "The value should not be empty, but was an empty {} value",
                            value_type(&base_value),
                        ),
                    },
                ))
            } else {
                Ok(())
            }
        }
    }
}
