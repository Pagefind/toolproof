use nondestructive::yaml;

use crate::{ToolproofTestFile, ToolproofTestStep};

pub fn write_yaml_snapshots(input_doc: &str, hydrated_file: &ToolproofTestFile) -> String {
    let mut doc = yaml::from_slice(input_doc).expect("Input doc parses as YAML");

    for (step_id, step) in hydrated_file.steps.iter().enumerate() {
        match step {
            ToolproofTestStep::Snapshot {
                snapshot_content, ..
            } => {
                let Some(snapshot_content) = snapshot_content else {
                    continue;
                };

                let mut step = doc
                    .as_mut()
                    .into_mapping_mut()
                    .unwrap()
                    .get_into_mut("steps")
                    .unwrap()
                    .into_sequence_mut()
                    .unwrap()
                    .get_into_mut(step_id)
                    .unwrap()
                    .into_mapping_mut()
                    .unwrap();

                step.insert_block(
                    "snapshot_content",
                    snapshot_content.lines().map(|l| format!("  ╎{l}")),
                    yaml::Block::Literal(yaml::Chomp::Strip),
                );
            }
            _ => {}
        }
    }

    doc.to_string()
}
