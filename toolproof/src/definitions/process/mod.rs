use std::collections::HashMap;

use async_trait::async_trait;

use crate::civilization::Civilization;
use crate::errors::{ToolproofInputError, ToolproofStepError};

use super::{SegmentArgs, ToolproofAssertion, ToolproofInstruction, ToolproofRetriever};

mod env_var {
    use super::*;

    pub struct EnvVar;

    inventory::submit! {
        &EnvVar as &dyn ToolproofInstruction
    }

    #[async_trait]
    impl ToolproofInstruction for EnvVar {
        fn segments(&self) -> &'static str {
            "I have the environment variable {name} set to {value}"
        }

        async fn run(
            &self,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<(), ToolproofStepError> {
            let env_name = args.get_string("name")?;
            let env_value = args.get_string("value")?;

            civ.set_env(env_name.to_string(), env_value.to_string());

            Ok(())
        }
    }
}

mod run {
    use crate::errors::ToolproofTestFailure;

    use super::*;

    pub struct Run;

    inventory::submit! {
        &Run as &dyn ToolproofInstruction
    }

    #[async_trait]
    impl ToolproofInstruction for Run {
        fn segments(&self) -> &'static str {
            "I run {command}"
        }

        async fn run(
            &self,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<(), ToolproofStepError> {
            let command = args.get_string("command")?;

            let exit_status = civ.run_command(command.to_string()).await?;

            if !exit_status.success() {
                return Err(ToolproofTestFailure::Custom {
                    msg: format!("Failed to run command ({})\nCommand: {command}\nstdout:\n---\n{}\n---\nstderr:\n---\n{}\n---",
                    exit_status,
                    civ.last_command_output.as_ref().map(|o| o.stdout.as_str()).unwrap_or_else(|| "<empty>"),
                    civ.last_command_output.as_ref().map(|o| o.stderr.as_str()).unwrap_or_else(|| "<empty>"),
                ),
                }
                .into());
            }

            Ok(())
        }
    }

    pub struct FailingRun;

    inventory::submit! {
        &FailingRun as &dyn ToolproofInstruction
    }

    #[async_trait]
    impl ToolproofInstruction for FailingRun {
        fn segments(&self) -> &'static str {
            "I run {command} and expect it to fail"
        }

        async fn run(
            &self,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<(), ToolproofStepError> {
            let command = args.get_string("command")?;

            let exit_status = civ.run_command(command.to_string()).await?;

            if exit_status.success() {
                return Err(ToolproofTestFailure::Custom {
                    msg: format!(
                        "Command ran successfully, but should not have ({})\nCommand: {command}\nstdout:\n---\n{}\n---\nstderr:\n---\n{}\n---",
                        exit_status,
                        civ.last_command_output.as_ref().map(|o| o.stdout.as_str()).unwrap_or_else(|| "<empty>"),
                        civ.last_command_output.as_ref().map(|o| o.stderr.as_str()).unwrap_or_else(|| "<empty>"),
                    ),
                }
                .into());
            }

            Ok(())
        }
    }
}

mod stdio {
    use crate::errors::ToolproofTestFailure;

    use super::*;

    pub struct StdOut;

    inventory::submit! {
        &StdOut as &dyn ToolproofRetriever
    }

    #[async_trait]
    impl ToolproofRetriever for StdOut {
        fn segments(&self) -> &'static str {
            "stdout"
        }

        async fn run(
            &self,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<serde_json::Value, ToolproofStepError> {
            let Some(output) = &civ.last_command_output else {
                return Err(ToolproofStepError::Assertion(
                    ToolproofTestFailure::Custom {
                        msg: "no stdout exists".into(),
                    },
                ));
            };

            Ok(output.stdout.clone().into())
        }
    }

    pub struct StdErr;

    inventory::submit! {
        &StdErr as &dyn ToolproofRetriever
    }

    #[async_trait]
    impl ToolproofRetriever for StdErr {
        fn segments(&self) -> &'static str {
            "stderr"
        }

        async fn run(
            &self,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<serde_json::Value, ToolproofStepError> {
            let Some(output) = &civ.last_command_output else {
                return Err(ToolproofStepError::Assertion(
                    ToolproofTestFailure::Custom {
                        msg: "no stderr exists".into(),
                    },
                ));
            };

            Ok(output.stderr.clone().into())
        }
    }
}
