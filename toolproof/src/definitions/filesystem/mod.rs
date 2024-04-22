use std::collections::HashMap;

use async_trait::async_trait;

use crate::civilization::Civilization;
use crate::errors::{ToolproofInputError, ToolproofStepError};

use super::{SegmentArgs, ToolproofInstruction, ToolproofRetriever};

mod new_file {

    use super::*;

    pub struct NewFile;

    inventory::submit! {
        &NewFile as &dyn ToolproofInstruction
    }

    #[async_trait]
    impl ToolproofInstruction for NewFile {
        fn segments(&self) -> &'static str {
            "I have a {filename} file with the content {contents}"
        }

        async fn run(
            &self,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<(), ToolproofStepError> {
            let filename = args.get_string("filename")?;
            if filename.is_empty() {
                return Err(ToolproofInputError::ArgumentRequiresValue {
                    arg: "filename".to_string(),
                }
                .into());
            }

            let contents = args.get_string("contents")?;

            civ.write_file(&filename, &contents);

            Ok(())
        }
    }
}

mod read_files {
    use crate::errors::ToolproofTestFailure;

    use super::*;

    pub struct PlainFile;

    inventory::submit! {
        &PlainFile as &dyn ToolproofRetriever
    }

    #[async_trait]
    impl ToolproofRetriever for PlainFile {
        fn segments(&self) -> &'static str {
            "The file {filename}"
        }

        async fn run(
            &self,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<serde_json::Value, ToolproofStepError> {
            let filename = args.get_string("filename")?;

            if filename.is_empty() {
                return Err(ToolproofInputError::ArgumentRequiresValue {
                    arg: "filename".to_string(),
                }
                .into());
            }

            let contents = civ.read_file(&filename)?;

            Ok(serde_json::Value::String(contents))
        }
    }
}
