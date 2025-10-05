use std::collections::BTreeMap;
use std::fmt::Display;
use std::ops::Div;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use std::{collections::HashMap, time::Instant};

use console::{style, Term};
use futures::future::join_all;
use miette::IntoDiagnostic;
use normalize_path::NormalizePath;
use parser::{parse_macro, ToolproofFileType, ToolproofPlatform};
use schematic::color::owo::OwoColorize;
use segments::ToolproofSegments;
use semver::{Version, VersionReq};
use similar_string::compare_similarity;
use tokio::fs::read_to_string;
use tokio::process::Command;
use tokio::sync::OnceCell;
use wax::Glob;

use crate::definitions::{register_assertions, register_instructions, register_retrievers};
use crate::differ::diff_snapshots;
use crate::errors::{ToolproofInputError, ToolproofStepError, ToolproofTestError};
use crate::interactive::{confirm_snapshot, get_run_mode, question, RunMode};
use crate::logging::log_step_runs;
use crate::options::configure;
use crate::parser::parse_segments;
use crate::universe::Universe;
use crate::{
    parser::parse_file, runner::run_toolproof_experiment, snapshot_writer::write_yaml_snapshots,
};

mod civilization;
mod definitions;
mod differ;
mod errors;
mod interactive;
mod logging;
mod options;
mod parser;
mod platforms;
mod runner;
mod segments;
mod snapshot_writer;
mod universe;

#[derive(Debug, Clone)]
pub struct ToolproofTestFile {
    pub name: String,
    r#type: ToolproofFileType,
    pub platforms: Option<Vec<ToolproofPlatform>>,
    pub steps: Vec<ToolproofTestStep>,
    pub original_source: String,
    pub file_path: String,
    pub file_directory: String,
    pub failure_screenshot: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ToolproofMacroFile {
    pub macro_segments: ToolproofSegments,
    pub macro_orig: String,
    pub steps: Vec<ToolproofTestStep>,
    pub original_source: String,
    pub file_path: String,
    pub file_directory: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolproofTestSuccess {
    Skipped,
    Passed { attempts: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolproofTestStepState {
    Dormant,
    Skipped,
    Failed,
    Passed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolproofTestStep {
    Ref {
        other_file: String,
        orig: String,
        hydrated_steps: Option<Vec<ToolproofTestStep>>,
        state: ToolproofTestStepState,
        platforms: Option<Vec<ToolproofPlatform>>,
    },
    Macro {
        step_macro: ToolproofSegments,
        args: HashMap<String, serde_json::Value>,
        orig: String,
        hydrated_steps: Option<Vec<ToolproofTestStep>>,
        state: ToolproofTestStepState,
        platforms: Option<Vec<ToolproofPlatform>>,
    },
    Instruction {
        step: ToolproofSegments,
        args: HashMap<String, serde_json::Value>,
        orig: String,
        state: ToolproofTestStepState,
        platforms: Option<Vec<ToolproofPlatform>>,
    },
    Assertion {
        retrieval: ToolproofSegments,
        assertion: ToolproofSegments,
        args: HashMap<String, serde_json::Value>,
        orig: String,
        state: ToolproofTestStepState,
        platforms: Option<Vec<ToolproofPlatform>>,
    },
    Snapshot {
        snapshot: ToolproofSegments,
        snapshot_content: Option<String>,
        args: HashMap<String, serde_json::Value>,
        orig: String,
        state: ToolproofTestStepState,
        platforms: Option<Vec<ToolproofPlatform>>,
    },
    Extract {
        extract: ToolproofSegments,
        extract_location: String,
        args: HashMap<String, serde_json::Value>,
        orig: String,
        state: ToolproofTestStepState,
        platforms: Option<Vec<ToolproofPlatform>>,
    },
}

impl Display for ToolproofTestStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ToolproofTestStep::*;

        match self {
            Instruction { orig, .. } | Assertion { orig, .. } => {
                write!(f, "{}", orig)
            }
            Macro { orig, .. } => {
                write!(f, "run steps from macro: {}", orig)
            }
            Ref { orig, .. } => {
                write!(f, "run steps from file: {}", orig)
            }
            Snapshot { orig, .. } => {
                write!(f, "snapshot: {}", orig)
            }
            Extract { orig, .. } => {
                write!(f, "extract: {}", orig)
            }
        }
    }
}

impl ToolproofTestStep {
    pub fn args_pretty(&self) -> String {
        let args = match self {
            ToolproofTestStep::Instruction { args, .. } => Some(args),
            ToolproofTestStep::Assertion { args, .. } => Some(args),
            ToolproofTestStep::Snapshot { args, .. } => Some(args),
            _ => None,
        };

        if let Some(args) = args {
            let res = format!("{}", serde_yaml::to_string(&args).unwrap());
            if res.trim() == "{}" {
                String::new()
            } else {
                res
            }
        } else {
            String::new()
        }
    }

    pub fn state(&self) -> ToolproofTestStepState {
        use ToolproofTestStep::*;

        match self {
            Ref { state, .. }
            | Macro { state, .. }
            | Instruction { state, .. }
            | Assertion { state, .. }
            | Extract { state, .. }
            | Snapshot { state, .. } => state.clone(),
        }
    }
}

fn closest_strings<'o>(target: &String, options: &'o Vec<String>) -> Vec<(&'o String, f64)> {
    let mut scores = options
        .iter()
        .map(|s| (s, compare_similarity(target, s)))
        .collect::<Vec<_>>();

    scores.sort_by(|a, b| {
        b.partial_cmp(a)
            .expect("similarities should not be NaN or Infinity")
    });

    scores
}

async fn main_inner() -> Result<(), ()> {
    let ctx = configure();

    if let Some(versions) = &ctx.params.supported_versions {
        let req = VersionReq::parse(versions).into_diagnostic().map_err(|e| {
            eprintln!("Failed to parse supported versions: {e:?}");
        })?;
        let active = Version::parse(&ctx.version).expect("Crate version should be valid");
        let is_local = ctx.version == "0.0.0";

        if !req.matches(&active) && !is_local {
            eprintln!(
                "Toolproof is running version {}, but your configuration requires Toolproof {}",
                ctx.version, versions
            );
            return Err(());
        }
    }

    if ctx.params.skip_hooks {
        println!("{}", "Skipping before_all commands".yellow().bold());
    } else {
        for before in &ctx.params.before_all {
            let before_cmd = &before.command;
            let mut command = Command::new("sh");
            command
                .arg("-c")
                .current_dir(&ctx.working_directory)
                .arg(before_cmd);

            command.stdout(Stdio::piped());
            command.stderr(Stdio::piped());

            println!(
                "{}{}",
                "Running before_all command: ".blue().bold(),
                before_cmd.cyan().bold(),
            );

            let running = command
                .spawn()
                .map_err(|_| eprintln!("Failed to run command: {before_cmd}"))?;

            let Ok(_) =
                (match tokio::time::timeout(Duration::from_secs(300), running.wait_with_output())
                    .await
                {
                    Ok(out) => out,
                    Err(_) => {
                        eprintln!("Failed to run command due to timeout: {before_cmd}");
                        return Err(());
                    }
                })
            else {
                eprintln!("Failed to run command: {before_cmd}");
                return Err(());
            };
        }
    }

    let start = Instant::now();

    let mut errors = vec![];

    let macro_glob = Glob::new("**/*.toolproof.macro.yml").expect("Valid glob");
    let macro_walker = macro_glob
        .walk(ctx.params.root.clone().unwrap_or(".".into()))
        .flatten();

    let loaded_macros = macro_walker
        .map(|entry| {
            let file = entry.path().to_path_buf();
            async { (file.clone(), read_to_string(file).await) }
        })
        .collect::<Vec<_>>();

    let macros = join_all(loaded_macros).await;

    let all_macros: HashMap<_, _> = macros
        .into_iter()
        .filter_map(|(p, i)| match parse_macro(&i.unwrap(), p.clone()) {
            Ok(f) => Some((f.macro_segments.clone(), f)),
            Err(e) => {
                errors.push(e);
                return None;
            }
        })
        .collect();

    let glob = Glob::new("**/*.toolproof.yml").expect("Valid glob");
    let walker = glob
        .walk(ctx.params.root.clone().unwrap_or(".".into()))
        .flatten();

    let loaded_files = walker
        .map(|entry| {
            let file = entry.path().to_path_buf();
            async { (file.clone(), read_to_string(file).await) }
        })
        .collect::<Vec<_>>();

    let files = join_all(loaded_files).await;

    let mut names_thus_far: Vec<(String, String)> = vec![];

    let all_tests: BTreeMap<_, _> = files
        .into_iter()
        .filter_map(|(p, i)| {
            let test_file = match parse_file(&i.unwrap(), p.clone()) {
                Ok(f) => {
                    if let Some((_, other_path)) = names_thus_far.iter().find(|(n, _)| *n == f.name)
                    {
                        errors.push(ToolproofInputError::DuplicateName {
                            path_one: other_path.to_string(),
                            path_two: p.to_string_lossy().to_string(),
                            name: f.name.clone(),
                        });
                        return None;
                    }
                    names_thus_far.push((f.name.clone(), p.to_string_lossy().to_string()));
                    f
                }
                Err(e) => {
                    errors.push(e);
                    return None;
                }
            };
            Some((p.normalize().to_string_lossy().into_owned(), test_file))
        })
        .collect();

    if !errors.is_empty() {
        eprintln!("Toolproof failed to parse some files:");
        for e in errors {
            eprintln!("  • {e}");
        }
        return Err(());
    }

    let macro_comparisons: Vec<_> = all_macros
        .keys()
        .map(|k| k.get_comparison_string())
        .collect();

    let all_instructions = register_instructions();
    let instruction_comparisons: Vec<_> = all_instructions
        .keys()
        .map(|k| k.get_comparison_string())
        .collect();

    let all_retrievers = register_retrievers();
    let retriever_comparisons: Vec<_> = all_retrievers
        .keys()
        .map(|k| k.get_comparison_string())
        .collect();

    let all_assertions = register_assertions();
    let assertion_comparisons: Vec<_> = all_assertions
        .keys()
        .map(|k| k.get_comparison_string())
        .collect();

    let universe = Arc::new(Universe {
        browser: OnceCell::new(),
        tests: all_tests,
        macros: all_macros,
        macro_comparisons,
        instructions: all_instructions,
        instruction_comparisons,
        retrievers: all_retrievers,
        retriever_comparisons,
        assertions: all_assertions,
        assertion_comparisons,
        ctx,
    });

    let run_mode = if let Some(run_name) = universe.ctx.params.run_name.as_ref() {
        let Some((path, _)) = universe.tests.iter().find(|(_, t)| t.name == *run_name) else {
            eprintln!("Test name {run_name} does not exist");
            return Err(());
        };

        RunMode::One(path.clone())
    } else if let Some(run_path) = universe.ctx.params.run_path.as_ref() {
        // Convert the provided path to an absolute path
        let absolute_path = if run_path.is_absolute() {
            run_path.clone()
        } else {
            universe.ctx.working_directory.join(run_path)
        };

        // Normalize the path for comparison
        let normalized_path = absolute_path.normalize();

        // Check if the path exists and is a file or directory
        if !absolute_path.exists() {
            eprintln!("Path does not exist: {}", run_path.display());
            return Err(());
        }

        RunMode::Path(normalized_path.to_string_lossy().into_owned())
    } else if universe.ctx.params.interactive && !universe.ctx.params.all {
        match get_run_mode(&universe) {
            Ok(mode) => mode,
            Err(e) => {
                eprintln!("{e}");
                return Err(());
            }
        }
    } else {
        RunMode::All
    };

    // Debugger mode requires running a single test
    if universe.ctx.params.debugger && !matches!(run_mode, RunMode::One(_)) {
        eprintln!(
            "Debugger mode requires running a single test. Please specify a test using --name."
        );
        return Err(());
    }

    // Validate that path-based filtering found at least one test
    if let RunMode::Path(ref filter_path) = run_mode {
        let test_root = universe.ctx.params.root.as_ref()
            .cloned()
            .unwrap_or_else(|| universe.ctx.working_directory.clone());

        let matching_tests = universe
            .tests
            .iter()
            .filter(|(test_path, v)| {
                if v.r#type != ToolproofFileType::Test {
                    return false;
                }

                // Convert relative test path to absolute for comparison
                let absolute_test_path = test_root.join(test_path).normalize();
                let absolute_test_path_str = absolute_test_path.to_string_lossy();

                absolute_test_path_str.as_ref() == filter_path || absolute_test_path_str.starts_with(filter_path.as_str())
            })
            .count();

        if matching_tests == 0 {
            eprintln!(
                "No tests found matching path: {}",
                universe.ctx.params.run_path.as_ref().unwrap().display()
            );
            return Err(());
        }
    }

    enum HoldingError {
        TestFailure,
        SnapFailure { out: String },
    }

    let handle_res = |universe: Arc<Universe>,
                      (file, res): (
        &ToolproofTestFile,
        Result<ToolproofTestSuccess, ToolproofTestError>,
    ),
                      started_at: Instant|
     -> Result<ToolproofTestSuccess, HoldingError> {
        let dur = if universe.ctx.params.porcelain {
            "".to_string()
        } else {
            let e = started_at.elapsed();
            format!("[{}.{:03}s] ", e.as_secs(), e.subsec_millis())
        };

        let log_err_preamble = || {
            println!(
                "{}",
                format!(
                    "{}{}{}",
                    "✘ ".red().bold(),
                    dur.red().bold().dimmed(),
                    &file.name.red().bold()
                )
            );
            println!("{}", style("--- STEPS ---").on_yellow().bold());
            log_step_runs(&file.steps, 0);
        };

        let output_doc = write_yaml_snapshots(&file.original_source, &file);

        match res {
            Ok(success) => {
                match success {
                    ToolproofTestSuccess::Skipped => {
                        let msg = format!(
                            "{}{}{}",
                            "⊝ ".green(),
                            dur.green().dimmed(),
                            &file.name.green()
                        );
                        println!("{}", style(msg).dim());
                        return Ok(success);
                    }
                    ToolproofTestSuccess::Passed { .. } => { /* continue to standard logging */ }
                }
                if output_doc.trim() == file.original_source.trim() {
                    let msg = format!(
                        "{}{}{}",
                        "✓ ".green(),
                        dur.green().dimmed(),
                        &file.name.green()
                    );
                    println!("{}", msg.green());
                    Ok(success)
                } else {
                    println!(
                        "{}",
                        format!(
                            "{}{}{}",
                            "⚠ ".yellow().bold(),
                            dur.yellow().bold().dimmed(),
                            &file.name.yellow().bold()
                        )
                    );
                    if !universe.ctx.params.interactive {
                        println!("{}\n", "--- SNAPSHOT CHANGED ---".on_bright_yellow().bold());
                        println!("{}", diff_snapshots(&file.original_source, &output_doc));
                        println!(
                            "\n{}",
                            "--- END SNAPSHOT CHANGE ---".on_bright_yellow().bold()
                        );
                        println!(
                            "\n{}",
                            "Run in interactive mode (-i) to accept new snapshots\n"
                                .bright_red()
                                .bold()
                        );
                    }
                    Err(HoldingError::SnapFailure { out: output_doc })
                }
            }
            Err(e) => {
                let log_err = || {
                    log_err_preamble();
                    println!("{}", "--- ERROR ---".on_yellow().bold());
                    println!("{}", &e.red());
                };

                let log_closest = |step_type: &str,
                                   original_segment_string: &str,
                                   user_segments: &ToolproofSegments,
                                   comparisons: &Vec<String>| {
                    let comparator = user_segments.get_comparison_string();

                    let matches = closest_strings(&comparator, comparisons);

                    eprintln!(
                        "Unable to resolve: \"{}\"\n{step_type} \"{}\" was not found.",
                        original_segment_string.red(),
                        comparator.yellow(),
                    );

                    matches
                        .into_iter()
                        .enumerate()
                        .filter_map(|(i, (s, score))| {
                            if i > 5 && score < 0.6 {
                                None
                            } else if i > 0 && score < 0.4 {
                                None
                            } else {
                                Some(parse_segments(&s).unwrap())
                            }
                        })
                        .collect::<Vec<_>>()
                };

                match &e.err {
                    ToolproofStepError::External(ex) => match ex {
                        errors::ToolproofInputError::NonexistentStep => {
                            log_err_preamble();
                            println!("{}", "--- ERROR ---".on_yellow().bold());
                            match &e.step {
                                ToolproofTestStep::Ref { .. } => println!("{}", &e.red()),
                                ToolproofTestStep::Macro {
                                    step_macro, orig, ..
                                } => {
                                    let closest = log_closest(
                                        "Macro",
                                        &orig,
                                        &step_macro,
                                        &universe.macro_comparisons,
                                    );

                                    let matches = closest
                                        .into_iter()
                                        .map(|m| {
                                            let (actual_segments, _) = universe
                                                .macros
                                                .get_key_value(&m)
                                                .expect("should exist in the global set");
                                            format!(
                                                "• {}",
                                                style(actual_segments.get_as_string()).cyan()
                                            )
                                        })
                                        .collect::<Vec<_>>();

                                    if matches.is_empty() {
                                        eprintln!("{}", "No similar macro found".red());
                                    } else {
                                        eprintln!("Closest macro:\n{}", matches.join("\n"));
                                    }
                                }
                                ToolproofTestStep::Instruction { step, orig, .. } => {
                                    let closest = log_closest(
                                        "Instruction",
                                        &orig,
                                        &step,
                                        &universe.instruction_comparisons,
                                    );

                                    let matches = closest
                                        .into_iter()
                                        .map(|m| {
                                            let (actual_segments, _) = universe
                                                .instructions
                                                .get_key_value(&m)
                                                .expect("should exist in the global set");
                                            format!(
                                                "• {}",
                                                style(actual_segments.get_as_string()).cyan()
                                            )
                                        })
                                        .collect::<Vec<_>>();

                                    if matches.is_empty() {
                                        eprintln!("{}", "No similar instructions found".red());
                                    } else {
                                        eprintln!("Closest instructions:\n{}", matches.join("\n"));
                                    }
                                }
                                ToolproofTestStep::Assertion {
                                    retrieval,
                                    assertion,
                                    orig,
                                    ..
                                } => {
                                    if !universe.retrievers.contains_key(&retrieval) {
                                        let closest = log_closest(
                                            "Retrieval",
                                            &orig,
                                            &retrieval,
                                            &universe.retriever_comparisons,
                                        );

                                        let matches = closest
                                            .into_iter()
                                            .map(|m| {
                                                let (actual_segments, _) = universe
                                                    .retrievers
                                                    .get_key_value(&m)
                                                    .expect("should exist in the global set");
                                                format!(
                                                    "• {}",
                                                    style(actual_segments.get_as_string()).cyan()
                                                )
                                            })
                                            .collect::<Vec<_>>();

                                        if matches.is_empty() {
                                            eprintln!("{}", "No similar retrievals found".red());
                                        } else {
                                            eprintln!(
                                                "Closest retrievals:\n{}",
                                                matches.join("\n")
                                            );
                                        }
                                    } else {
                                        let closest = log_closest(
                                            "Assertion",
                                            &orig,
                                            &assertion,
                                            &universe.assertion_comparisons,
                                        );

                                        let matches = closest
                                            .into_iter()
                                            .map(|m| {
                                                let (actual_segments, _) = universe
                                                    .assertions
                                                    .get_key_value(&m)
                                                    .expect("should exist in the global set");
                                                format!(
                                                    "• {}",
                                                    style(actual_segments.get_as_string()).cyan()
                                                )
                                            })
                                            .collect::<Vec<_>>();

                                        if matches.is_empty() {
                                            eprintln!("{}", "No similar assertions found".red());
                                        } else {
                                            eprintln!(
                                                "Closest assertions:\n{}",
                                                matches.join("\n")
                                            );
                                        }
                                    }
                                }
                                ToolproofTestStep::Extract { .. } => todo!(),
                                ToolproofTestStep::Snapshot { .. } => todo!(),
                            }
                        }
                        _ => {
                            log_err();
                        }
                    },
                    _ => {
                        log_err();
                    }
                }

                if let Some(failure_screenshot) = &file.failure_screenshot {
                    println!("{}", "--- FAILURE SCREENSHOT ---".on_yellow().bold());
                    println!(
                        "{} {}",
                        "Browser state at failure was screenshot to".red(),
                        failure_screenshot.to_string_lossy().cyan().bold()
                    );
                }

                Err(HoldingError::TestFailure)
            }
        }
    };

    let semaphore = Arc::new(tokio::sync::Semaphore::new(universe.ctx.params.concurrency));

    let mut hands = vec![];

    println!("\n{}\n", "Running tests".bold());

    match run_mode {
        RunMode::All => {
            for mut test in universe
                .tests
                .values()
                .filter(|v| v.r#type == ToolproofFileType::Test)
                .cloned()
            {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let uni = Arc::clone(&universe);
                hands.push(tokio::spawn(async move {
                    let start = Instant::now();
                    let res = run_toolproof_experiment(&mut test, Arc::clone(&uni)).await;
                    let holding_err = handle_res(uni, (&test, res), start);

                    drop(permit);

                    holding_err.map_err(|e| (test, e))
                }));
            }
        }
        RunMode::One(t) => {
            let mut test = universe.tests.get(&t).cloned().unwrap();
            let uni = Arc::clone(&universe);
            hands.push(tokio::spawn(async move {
                let start = Instant::now();
                let res = run_toolproof_experiment(&mut test, Arc::clone(&uni)).await;
                let holding_err = handle_res(uni, (&test, res), start);

                holding_err.map_err(|e| (test, e))
            }));
        }
        RunMode::Path(ref filter_path) => {
            let test_root = universe.ctx.params.root.as_ref()
                .cloned()
                .unwrap_or_else(|| universe.ctx.working_directory.clone());

            for mut test in universe
                .tests
                .iter()
                .filter(|(test_path, v)| {
                    if v.r#type != ToolproofFileType::Test {
                        return false;
                    }

                    // Convert relative test path to absolute for comparison
                    let absolute_test_path = test_root.join(test_path).normalize();
                    let absolute_test_path_str = absolute_test_path.to_string_lossy();

                    absolute_test_path_str.as_ref() == filter_path || absolute_test_path_str.starts_with(filter_path.as_str())
                })
                .map(|(_, v)| v.clone())
            {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let uni = Arc::clone(&universe);
                hands.push(tokio::spawn(async move {
                    let start = Instant::now();
                    let res = run_toolproof_experiment(&mut test, Arc::clone(&uni)).await;
                    let holding_err = handle_res(uni, (&test, res), start);

                    drop(permit);

                    holding_err.map_err(|e| (test, e))
                }));
            }
        }
    }

    let mut results = join_all(hands)
        .await
        .into_iter()
        .map(|outer_err| match outer_err {
            Ok(Ok(success)) => Ok(success),
            Ok(Err(e)) => Err(e),
            Err(e) => panic!("Failed to await all tests: {e}"),
        })
        .collect::<Vec<_>>();

    let retry_count = universe.ctx.params.retry_count;
    let mut concurrency = universe.ctx.params.concurrency;
    for i in 0..retry_count {
        if !results.iter().any(|r| r.is_err()) {
            break;
        }

        let remaining_attempts = retry_count - i;
        concurrency = concurrency.div(2).max(1);
        println!(
            "{}",
            style(&format!(
                "\nSome tests failed. Retrying {} at concurrency {concurrency}.",
                if remaining_attempts == 1 {
                    "once".to_string()
                } else {
                    format!("{remaining_attempts} times")
                }
            ))
            .yellow()
        );

        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));
        let mut hands = vec![];

        for (result_index, result) in results.iter().enumerate().filter(|(_, r)| r.is_err()) {
            if let Err((test, _)) = result {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let uni = Arc::clone(&universe);
                let mut new_test = test.clone();
                hands.push(tokio::spawn(async move {
                    let start = Instant::now();
                    let res = run_toolproof_experiment(&mut new_test, Arc::clone(&uni)).await;
                    let holding_err = handle_res(uni, (&new_test, res), start);

                    drop(permit);

                    (
                        result_index,
                        holding_err.map_err(|e| (new_test, e)).map(|r| {
                            if matches!(r, ToolproofTestSuccess::Passed { .. }) {
                                ToolproofTestSuccess::Passed { attempts: i + 1 }
                            } else {
                                r
                            }
                        }),
                    )
                }));
            }
        }

        for (result_index, retried_result) in
            join_all(hands)
                .await
                .into_iter()
                .filter_map(|outer_err| match outer_err {
                    Ok((i, Ok(success))) => Some((i, success)),
                    _ => None,
                })
        {
            results[result_index] = Ok(retried_result);
        }
    }

    let snapshot_failures = results
        .iter()
        .filter_map(|r| match r {
            Err((f, HoldingError::SnapFailure { out })) => Some((f, out)),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut resolved_errors = 0;

    println!("\n{}\n", "Finished running tests".bold());

    let interactive = universe.ctx.params.interactive;
    if interactive && !snapshot_failures.is_empty() {
        let review_snapshots = match question(format!(
            "{} {}. Review now?",
            snapshot_failures.len(),
            if snapshot_failures.len() == 1 {
                "snapshot has changed"
            } else {
                "snapshots have changed"
            },
        )) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("{e}");
                return Err(());
            }
        };

        if review_snapshots {
            let term = Term::stdout();

            for (file, failure) in results.iter().filter_map(|r| match r {
                Ok(_) => None,
                Err(e) => Some(e),
            }) {
                match failure {
                    HoldingError::TestFailure => {}
                    HoldingError::SnapFailure { out } => {
                        if confirm_snapshot(&term, &file, &out).is_ok_and(|v| v) {
                            resolved_errors += 1;

                            if let Err(e) = tokio::fs::write(&file.file_path, out).await {
                                eprintln!("Unable to write updates snapshot to disk.\n{e}");
                                return Err(());
                            }
                        }
                    }
                }
            }
            println!("\n\n");
        }
    }

    let duration = start.elapsed();
    let duration = if universe.ctx.params.porcelain {
        "".to_string()
    } else {
        format!(
            " in {}.{:03} seconds",
            duration.as_secs(),
            duration.subsec_millis()
        )
    };

    let failing = results.iter().filter(|r| r.is_err()).count() - resolved_errors;
    let passing = results
        .iter()
        .filter(|r| matches!(r, Ok(ToolproofTestSuccess::Passed { .. })))
        .count()
        + resolved_errors;
    let skipped = results
        .iter()
        .filter(|r| matches!(r, Ok(ToolproofTestSuccess::Skipped)))
        .count();

    let retried_passed = if universe.ctx.params.retry_count > 0 {
        results
            .iter()
            .filter(|r| matches!(r, Ok(ToolproofTestSuccess::Passed { attempts: run }) if *run > 0))
            .count()
    } else {
        0
    };

    println!(
        "{}\n{}\n{}\n{}",
        style(&format!("Total passing tests: {}", passing)).cyan(),
        style(&format!("Passed after retry: {}", retried_passed)).cyan(),
        style(&format!("Failing tests: {}", failing)).cyan(),
        style(&format!("Skipped tests: {}", skipped)).cyan(),
    );

    if failing > 0 {
        println!(
            "{}",
            style(&format!("\nSome tests failed{}", duration)).red()
        );
        return Err(());
    } else {
        println!(
            "{}",
            style(&format!("\nAll tests passed{}", duration)).green()
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    match main_inner().await {
        Ok(_) => std::process::exit(0),
        Err(_) => std::process::exit(1),
    }
}
