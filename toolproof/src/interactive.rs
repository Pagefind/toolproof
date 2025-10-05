use std::{io, sync::Arc};

use console::{Key, Term};
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Select};
use schematic::color::owo::OwoColorize;

use crate::{
    differ::diff_snapshots, errors::ToolproofInternalError, parser::ToolproofFileType,
    universe::Universe, ToolproofTestFile,
};

#[derive(Debug)]
pub enum RunMode {
    All,
    One(String),
    Path(String),
}

impl From<dialoguer::Error> for ToolproofInternalError {
    fn from(value: dialoguer::Error) -> Self {
        ToolproofInternalError::Custom {
            msg: format!(
                "Failed to read the interactive terminal,\
                 try running in non-interactive mode.\n\
                 Source error: {value}"
            ),
        }
    }
}
impl From<io::Error> for ToolproofInternalError {
    fn from(value: io::Error) -> Self {
        ToolproofInternalError::Custom {
            msg: format!(
                "Failed to read the interactive terminal,\
                 try running in non-interactive mode.\n\
                 Source error: {value}"
            ),
        }
    }
}

pub fn get_run_mode(universe: &Arc<Universe>) -> Result<RunMode, ToolproofInternalError> {
    println!("{}\n", "Welcome to toolproof!".bold());

    let mode = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Which tests do you want to run?")
        .items(&["All tests", "Select test"])
        .interact()?;
    if mode == 0 {
        return Ok(RunMode::All);
    }

    let tests = universe
        .tests
        .iter()
        .filter(|(_, v)| v.r#type == ToolproofFileType::Test)
        .map(|(k, v)| (k, &v.name))
        .collect::<Vec<_>>();
    let test_names = tests
        .iter()
        .map(|(path, name)| format!("{} ({})", path, name))
        .collect::<Vec<_>>();

    let test = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Which test do you want to run? (type to filter)")
        .items(&test_names)
        .interact()?;

    Ok(RunMode::One(tests[test].0.clone()))
}

pub fn question(s: impl AsRef<str>) -> Result<bool, ToolproofInternalError> {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(s.as_ref())
        .interact()
        .map_err(Into::into)
}

pub fn confirm_snapshot(
    term: &Term,
    file: &ToolproofTestFile,
    new: &str,
) -> Result<bool, ToolproofInternalError> {
    /* ============================================
     * TOMO-TODO
     * have diff_snapshots return a string.
     * calculate the height and move down the screen before printing.
     * add a char to turn the diff off and print the full `new` str
     * add command instructions to the bottom of the screen
     */

    let diffed_snap = diff_snapshots(&file.original_source, new);
    let snap_height = diffed_snap.lines().count();
    let source_height = new.lines().count();
    let mut render_diff = true;

    println!(
        "\n- - - - - - - - - - - - - - -\n\n{}",
        "Reviewing snapshot".bold()
    );
    println!("File: {}", file.file_path.magenta());
    println!("Name: {}\n", file.name.cyan());

    let mut resp = None;
    while resp.is_none() {
        if render_diff {
            println!("{diffed_snap}");
        } else {
            println!("{}", &new.trim());
        };

        println!(
            "\n{} accept, {} reject, {} toggle diff",
            "[a]".green().bold(),
            "[r]".red().bold(),
            "[d]".cyan().bold(),
        );

        loop {
            match term.read_key()? {
                Key::Char('a') => {
                    resp = Some(true);
                    break;
                }
                Key::Char('r') => {
                    resp = Some(false);
                    break;
                }
                Key::Char('d') => {
                    if render_diff {
                        _ = term.clear_last_lines(snap_height + 2);
                    } else {
                        _ = term.clear_last_lines(source_height + 2);
                    }
                    render_diff = !render_diff;
                    break;
                }
                _ => {}
            }
        }
    }

    _ = term.clear_last_lines(2);
    let res = resp.unwrap_or_default();

    if res {
        println!("\n{}", "Accepted new snapshot, saving...".green().bold());
    } else {
        println!("\n{}", "Rejected snapshot.".red().bold());
    }

    Ok(res)
}
