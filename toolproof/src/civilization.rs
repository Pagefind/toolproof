use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::PathBuf,
    process::{ExitStatus, Stdio},
    str::from_utf8,
    sync::Arc,
    time::Duration,
};

use actix_web::dev::ServerHandle;
use portpicker::pick_unused_port;
use tempfile::tempdir;
use tokio::{process::Command, task::JoinHandle};
use wax::Glob;

use crate::{
    definitions::browser::BrowserWindow, errors::ToolproofTestFailure, universe::Universe,
};

#[derive(Debug)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
}

pub struct Civilization<'u> {
    pub tmp_dir: Option<tempfile::TempDir>,
    pub last_command_output: Option<CommandOutput>,
    pub assigned_server_port: Option<u16>,
    pub window: Option<BrowserWindow>,
    pub threads: Vec<JoinHandle<Result<(), std::io::Error>>>,
    pub handles: Vec<ServerHandle>,
    pub env_vars: HashMap<String, String>,
    pub universe: Arc<Universe<'u>>,
}

impl<'u> Civilization<'u> {
    pub async fn shutdown(&mut self) {
        for handle in &self.handles {
            handle.stop(false).await;
        }
        for thread in &self.threads {
            thread.abort();
        }
    }
}

impl<'u> Civilization<'u> {
    pub fn ensure_port(&mut self) -> u16 {
        if self.assigned_server_port.is_none() {
            self.assigned_server_port = pick_unused_port();
        }
        self.assigned_server_port.expect("No port was available")
    }
    pub fn purge_port(&mut self) {
        self.assigned_server_port = None;
    }

    pub fn tmp_dir(&mut self) -> PathBuf {
        if self.tmp_dir.is_none() {
            self.tmp_dir = Some(tempdir().expect("testing on a system with a temp dir"));
        }
        self.tmp_dir
            .as_ref()
            .expect("just created")
            .path()
            .to_path_buf()
    }

    pub fn tmp_file_path(&mut self, filename: &str) -> PathBuf {
        let tmp_dir = self.tmp_dir();
        tmp_dir.join(PathBuf::from(filename))
    }

    pub fn write_file(&mut self, filename: &str, contents: &str) {
        let file_path = self.tmp_file_path(filename);
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();

        // let contents = self.process_substitutions(contents);

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }

    pub fn read_file(&mut self, filename: &str) -> Result<String, ToolproofTestFailure> {
        let file_path = self.tmp_file_path(filename);
        let mut file = std::fs::File::open(&file_path).map_err(|e| {
            let msg = match e.kind() {
                std::io::ErrorKind::NotFound => "the file does not exist".to_string(),
                _ => "the file was not readable".to_string(),
            };
            ToolproofTestFailure::Custom { msg }
        })?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|_| ToolproofTestFailure::Custom {
                msg: "the file was not valid UTF-8".to_string(),
            })?;
        Ok(contents)
    }

    pub fn get_file_tree(&mut self) -> String {
        let glob = Glob::new("**/*").expect("Valid glob");
        let base_dir = self.tmp_file_path(".");
        let walk = glob.walk(&base_dir).flatten();
        let entries: Vec<String> = walk
            .filter_map(|entry| {
                let file = entry
                    .path()
                    .strip_prefix(&base_dir)
                    .expect("Valid file path");
                let indentation = "  ".repeat(file.components().count().saturating_sub(1));
                file.file_name().map(|filename| {
                    format!(
                        "| {}{}",
                        indentation,
                        filename.to_str().expect("Valid filename utf8")
                    )
                })
            })
            .collect();
        entries.join("\n")
    }

    pub fn assert_file_exists(&mut self, filename: &str) {
        if !self.check_file_exists(filename) {
            panic!(
                "\"{}\" does not exist in the tree:\n-----\n{}\n-----\n",
                filename,
                self.get_file_tree()
            );
        }
    }

    pub fn assert_file_doesnt_exist(&mut self, filename: &str) {
        if self.check_file_exists(filename) {
            panic!(
                "\"{}\" should not exist but does in the tree:\n-----\n{}\n-----\n",
                filename,
                self.get_file_tree()
            );
        }
    }

    pub fn check_file_exists(&mut self, filename: &str) -> bool {
        self.tmp_file_path(filename).exists()
    }

    pub fn set_env(&mut self, name: String, value: String) {
        self.env_vars.insert(name, value);
    }

    pub async fn run_command(&mut self, cmd: String) -> Result<ExitStatus, ToolproofTestFailure> {
        let mut command = Command::new("sh");
        command
            .arg("-c")
            .current_dir(self.tmp_dir())
            .arg(&cmd.replace(std::path::MAIN_SEPARATOR, "/"));

        for (key, val) in &self.env_vars {
            command.env(key, val);
        }

        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let running = command.spawn().map_err(|_| ToolproofTestFailure::Custom {
            msg: format!("Failed to run command: {cmd}"),
        })?;

        let Ok(output) = (match tokio::time::timeout(
            Duration::from_secs(30),
            running.wait_with_output(),
        )
        .await
        {
            Ok(out) => out,
            Err(_) => {
                return Err(ToolproofTestFailure::Custom {
                    msg: format!("Failed to run command due to timeout: {cmd}"),
                });
            }
        }) else {
            return Err(ToolproofTestFailure::Custom {
                msg: format!("Failed to run command: {cmd}"),
            });
        };

        self.last_command_output = Some(CommandOutput {
            stdout: from_utf8(&strip_ansi_escapes::strip(&output.stdout))
                .unwrap_or("failed utf8")
                .into(),
            stderr: from_utf8(&strip_ansi_escapes::strip(&output.stderr))
                .unwrap_or("failed utf8")
                .into(),
        });

        Ok(output.status)
    }
}
