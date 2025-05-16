use async_recursion::async_recursion;
use futures::FutureExt;
use normalize_path::NormalizePath;
use similar_string::find_best_similarity;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::time::{self, Duration};

use console::style;

use crate::{
    civilization::Civilization,
    definitions::{browser::screenshots::ScreenshotViewport, ToolproofInstruction},
    errors::{ToolproofInputError, ToolproofStepError, ToolproofTestError, ToolproofTestFailure},
    platforms::platform_matches,
    segments::SegmentArgs,
    universe::Universe,
    ToolproofTestFile, ToolproofTestStep, ToolproofTestStepState, ToolproofTestSuccess,
};

pub async fn run_toolproof_experiment(
    input: &mut ToolproofTestFile,
    universe: Arc<Universe<'_>>,
) -> Result<ToolproofTestSuccess, ToolproofTestError> {
    if !platform_matches(&input.platforms) {
        return Ok(ToolproofTestSuccess::Skipped);
    }

    let mut civ = Civilization {
        tmp_dir: None,
        last_command_output: None,
        assigned_server_port: None,
        window: None,
        threads: vec![],
        handles: vec![],
        env_vars: HashMap::new(),
        universe,
    };

    let res = run_toolproof_steps(&input.file_directory, &mut input.steps, &mut civ, None).await;

    if res.is_err() && civ.window.is_some() {
        if let Some(screenshot_target) = &civ.universe.ctx.params.failure_screenshot_location {
            let instruction = ScreenshotViewport {};
            let filename = format!(
                "{}-{}.webp",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Toolproof should be running after the UNIX EPOCH")
                    .as_secs(),
                input.file_path.replace(|c: char| !c.is_alphanumeric(), "-")
            );
            let abs_acreenshot_target = civ.universe.ctx.working_directory.join(screenshot_target);
            let filepath = abs_acreenshot_target.join(filename);
            if instruction
                .run(
                    &SegmentArgs::build_synthetic(
                        [(
                            "filepath".to_string(),
                            &serde_json::Value::String(filepath.to_string_lossy().to_string()),
                        )]
                        .into(),
                    ),
                    &mut civ,
                )
                .await
                .is_ok()
            {
                input.failure_screenshot = Some(filepath)
            }
        }
    }

    civ.shutdown().await;

    res
}

#[async_recursion]
async fn run_toolproof_steps(
    file_directory: &String,
    steps: &mut Vec<ToolproofTestStep>,
    civ: &mut Civilization<'_>,
    transient_placeholders: Option<HashMap<String, String>>,
) -> Result<ToolproofTestSuccess, ToolproofTestError> {
    let timeout_mins = civ.universe.ctx.params.timeout;
    let timeout_dur = Duration::from_secs(timeout_mins);
    for cur_step in steps.iter_mut() {
        let marked_base_step = cur_step.clone();
        let marked_base_args = cur_step.args_pretty();

        let mark_and_return_step_error =
            |e: ToolproofStepError, state: &mut ToolproofTestStepState| {
                *state = ToolproofTestStepState::Failed;
                ToolproofTestError {
                    err: e.into(),
                    step: marked_base_step.clone(),
                    arg_str: marked_base_args.clone(),
                }
            };
        let timeout_and_return_step_error = |state: &mut ToolproofTestStepState| {
            *state = ToolproofTestStepState::Failed;
            ToolproofTestError {
                err: ToolproofStepError::Assertion(ToolproofTestFailure::Custom {
                    msg: format!("Step timed out after {timeout_mins}s"),
                }),
                step: marked_base_step.clone(),
                arg_str: marked_base_args.clone(),
            }
        };

        match cur_step {
            crate::ToolproofTestStep::Ref {
                other_file,
                orig: _,
                hydrated_steps,
                state,
                platforms,
            } => {
                let target_path = PathBuf::from(file_directory)
                    .join(other_file)
                    .normalize()
                    .to_string_lossy()
                    .into_owned();
                let Some(target_file) = civ.universe.tests.get(&target_path).cloned() else {
                    let avail = civ.universe.tests.keys().collect::<Vec<_>>();
                    let closest = find_best_similarity(&target_path, &avail).map(|s| s.0);
                    return Err(mark_and_return_step_error(
                        ToolproofStepError::External(ToolproofInputError::InvalidRef {
                            input: target_path,
                            closest: closest.unwrap_or_else(|| "<nothing found>".to_string()),
                        }),
                        state,
                    ));
                };

                *hydrated_steps = Some(target_file.steps);

                if platform_matches(platforms) {
                    match run_toolproof_steps(
                        &target_file.file_directory,
                        hydrated_steps.as_mut().unwrap(),
                        civ,
                        None,
                    )
                    .await
                    {
                        Ok(_) => {
                            *state = ToolproofTestStepState::Passed;
                        }
                        Err(e) => {
                            *state = ToolproofTestStepState::Failed;
                            return Err(e);
                        }
                    }
                } else {
                    *state = ToolproofTestStepState::Skipped;
                }
            }
            crate::ToolproofTestStep::Macro {
                step_macro,
                args,
                orig: _,
                hydrated_steps,
                state,
                platforms,
            } => {
                let Some((reference_segments, defined_macro)) =
                    civ.universe.macros.get_key_value(step_macro)
                else {
                    *state = ToolproofTestStepState::Failed;
                    return Err(mark_and_return_step_error(
                        ToolproofStepError::External(ToolproofInputError::NonexistentStep),
                        state,
                    ));
                };
                let variable_names = reference_segments.get_variable_names();

                let defined_macro = defined_macro.clone();
                let macro_args = SegmentArgs::build(
                    reference_segments,
                    step_macro,
                    args,
                    Some(&civ),
                    transient_placeholders.as_ref(),
                )
                .map_err(|e| mark_and_return_step_error(e.into(), state))?;

                let mut macro_placeholders = HashMap::with_capacity(variable_names.len());
                for name in variable_names {
                    match macro_args.get_string(&name) {
                        Ok(res) => {
                            macro_placeholders.insert(name, res);
                        }
                        Err(e) => return Err(mark_and_return_step_error(e.into(), state)),
                    }
                }

                *hydrated_steps = Some(defined_macro.steps.clone());

                if platform_matches(platforms) {
                    match run_toolproof_steps(
                        &defined_macro.file_directory,
                        hydrated_steps.as_mut().unwrap(),
                        civ,
                        Some(macro_placeholders),
                    )
                    .await
                    {
                        Ok(_) => {
                            *state = ToolproofTestStepState::Passed;
                        }
                        Err(e) => {
                            *state = ToolproofTestStepState::Failed;
                            return Err(e);
                        }
                    }
                } else {
                    *state = ToolproofTestStepState::Skipped;
                }
            }
            crate::ToolproofTestStep::Instruction {
                step,
                args,
                orig,
                state,
                platforms,
            } => {
                let Some((reference_segments, instruction)) =
                    civ.universe.instructions.get_key_value(step)
                else {
                    *state = ToolproofTestStepState::Failed;
                    return Err(mark_and_return_step_error(
                        ToolproofStepError::External(ToolproofInputError::NonexistentStep),
                        state,
                    ));
                };

                let instruction_args = SegmentArgs::build(
                    reference_segments,
                    step,
                    args,
                    Some(&civ),
                    transient_placeholders.as_ref(),
                )
                .map_err(|e| mark_and_return_step_error(e.into(), state))?;

                if platform_matches(platforms) {
                    match time::timeout(timeout_dur, instruction.run(&instruction_args, civ)).await
                    {
                        Ok(Ok(_)) => {}
                        Ok(Err(e)) => {
                            return Err(mark_and_return_step_error(e.into(), state));
                        }
                        Err(_) => {
                            return Err(timeout_and_return_step_error(state));
                        }
                    }

                    *state = ToolproofTestStepState::Passed;
                } else {
                    *state = ToolproofTestStepState::Skipped;
                }
            }
            crate::ToolproofTestStep::Assertion {
                retrieval,
                assertion,
                args,
                orig,
                state,
                platforms,
            } => {
                let Some((reference_ret, retrieval_step)) =
                    civ.universe.retrievers.get_key_value(retrieval)
                else {
                    return Err(mark_and_return_step_error(
                        ToolproofStepError::External(ToolproofInputError::NonexistentStep),
                        state,
                    ));
                };

                let retrieval_args = SegmentArgs::build(
                    reference_ret,
                    retrieval,
                    args,
                    Some(&civ),
                    transient_placeholders.as_ref(),
                )
                .map_err(|e| mark_and_return_step_error(e.into(), state))?;

                let value = if platform_matches(platforms) {
                    match time::timeout(timeout_dur, retrieval_step.run(&retrieval_args, civ)).await
                    {
                        Ok(Ok(val)) => val,
                        Ok(Err(e)) => {
                            return Err(mark_and_return_step_error(e.into(), state));
                        }
                        Err(_) => {
                            return Err(timeout_and_return_step_error(state));
                        }
                    }
                } else {
                    serde_json::Value::Null
                };

                let Some((reference_assert, assertion_step)) =
                    civ.universe.assertions.get_key_value(assertion)
                else {
                    return Err(mark_and_return_step_error(
                        ToolproofStepError::External(ToolproofInputError::NonexistentStep),
                        state,
                    ));
                };

                let assertion_args = SegmentArgs::build(
                    reference_assert,
                    assertion,
                    args,
                    Some(&civ),
                    transient_placeholders.as_ref(),
                )
                .map_err(|e| mark_and_return_step_error(e.into(), state))?;

                if platform_matches(platforms) {
                    match time::timeout(
                        timeout_dur,
                        assertion_step.run(value, &assertion_args, civ),
                    )
                    .await
                    {
                        Ok(Ok(_)) => {}
                        Ok(Err(e)) => {
                            return Err(mark_and_return_step_error(e.into(), state));
                        }
                        Err(_) => {
                            return Err(timeout_and_return_step_error(state));
                        }
                    }

                    *state = ToolproofTestStepState::Passed;
                } else {
                    *state = ToolproofTestStepState::Skipped;
                }
            }
            crate::ToolproofTestStep::Snapshot {
                snapshot,
                snapshot_content,
                args,
                orig: _,
                state,
                platforms,
            } => {
                let Some((reference_ret, retrieval_step)) =
                    civ.universe.retrievers.get_key_value(snapshot)
                else {
                    return Err(mark_and_return_step_error(
                        ToolproofStepError::External(ToolproofInputError::NonexistentStep),
                        state,
                    ));
                };

                let retrieval_args = SegmentArgs::build(
                    reference_ret,
                    snapshot,
                    args,
                    Some(&civ),
                    transient_placeholders.as_ref(),
                )
                .map_err(|e| mark_and_return_step_error(e.into(), state))?;

                if platform_matches(platforms) {
                    let value =
                        match time::timeout(timeout_dur, retrieval_step.run(&retrieval_args, civ))
                            .await
                        {
                            Ok(Ok(val)) => val,
                            Ok(Err(e)) => {
                                return Err(mark_and_return_step_error(e.into(), state));
                            }
                            Err(_) => {
                                return Err(timeout_and_return_step_error(state));
                            }
                        };

                    let value_content = match &value {
                        serde_json::Value::String(s) => s.clone(),
                        _ => serde_yaml::to_string(&value).expect("snapshot value is serializable"),
                    };

                    *snapshot_content = Some(value_content);
                    *state = ToolproofTestStepState::Passed;
                } else {
                    *state = ToolproofTestStepState::Skipped;
                }
            }
            crate::ToolproofTestStep::Extract {
                extract,
                extract_location,
                args,
                orig: _,
                state,
                platforms,
            } => {
                let Some((reference_ret, retrieval_step)) =
                    civ.universe.retrievers.get_key_value(extract)
                else {
                    return Err(mark_and_return_step_error(
                        ToolproofStepError::External(ToolproofInputError::NonexistentStep),
                        state,
                    ));
                };

                let retrieval_args = SegmentArgs::build(
                    reference_ret,
                    extract,
                    args,
                    Some(&civ),
                    transient_placeholders.as_ref(),
                )
                .map_err(|e| mark_and_return_step_error(e.into(), state))?;

                if platform_matches(platforms) {
                    let value =
                        match time::timeout(timeout_dur, retrieval_step.run(&retrieval_args, civ))
                            .await
                        {
                            Ok(Ok(val)) => val,
                            Ok(Err(e)) => {
                                return Err(mark_and_return_step_error(e.into(), state));
                            }
                            Err(_) => {
                                return Err(timeout_and_return_step_error(state));
                            }
                        };

                    let value_content = match &value {
                        serde_json::Value::String(s) => s.clone(),
                        _ => serde_yaml::to_string(&value).expect("extract value is serializable"),
                    };

                    let location = retrieval_args.process_external_string(&extract_location);
                    civ.write_file(&location, &value_content);

                    *state = ToolproofTestStepState::Passed;
                } else {
                    *state = ToolproofTestStepState::Skipped;
                }
            }
        }
    }

    Ok(ToolproofTestSuccess::Passed { attempts: 0 })
}
