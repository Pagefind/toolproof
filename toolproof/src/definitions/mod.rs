use std::collections::HashMap;

use async_trait::async_trait;

use crate::{
    civilization::Civilization,
    errors::ToolproofStepError,
    parser::parse_segments,
    segments::{SegmentArgs, ToolproofSegments},
};

mod assertions;
pub mod browser;
mod filesystem;
mod hosting;
mod process;

/// Main instructions, generally start with "I ..."
#[async_trait]
pub trait ToolproofInstruction: Sync {
    fn segments(&self) -> &'static str;
    async fn run(
        &self,
        args: &SegmentArgs<'_>,
        civ: &mut Civilization,
    ) -> Result<(), ToolproofStepError>;
}

inventory::collect!(&'static dyn ToolproofInstruction);

pub fn register_instructions() -> HashMap<ToolproofSegments, &'static dyn ToolproofInstruction> {
    HashMap::<_, _>::from_iter(
        (inventory::iter::<&dyn ToolproofInstruction>)
            .into_iter()
            .map(|i| {
                let segments =
                    parse_segments(i.segments()).expect("builtin instructions should be parseable");

                (segments, *i)
            }),
    )
}

/// Retrievers, used before a "should" clause
#[async_trait]
pub trait ToolproofRetriever: Sync {
    fn segments(&self) -> &'static str;
    async fn run(
        &self,
        args: &SegmentArgs<'_>,
        civ: &mut Civilization,
    ) -> Result<serde_json::Value, ToolproofStepError>;
}

inventory::collect!(&'static dyn ToolproofRetriever);

pub fn register_retrievers() -> HashMap<ToolproofSegments, &'static dyn ToolproofRetriever> {
    HashMap::<_, _>::from_iter(
        (inventory::iter::<&dyn ToolproofRetriever>)
            .into_iter()
            .map(|i| {
                let segments =
                    parse_segments(i.segments()).expect("builtin retrievers should be parseable");

                (segments, *i)
            }),
    )
}

/// Assertions, used after a "should" clause
#[async_trait]
pub trait ToolproofAssertion: Sync {
    fn segments(&self) -> &'static str;
    async fn run(
        &self,
        base_value: serde_json::Value,
        args: &SegmentArgs<'_>,
        civ: &mut Civilization,
    ) -> Result<(), ToolproofStepError>;
}

inventory::collect!(&'static dyn ToolproofAssertion);

pub fn register_assertions() -> HashMap<ToolproofSegments, &'static dyn ToolproofAssertion> {
    HashMap::<_, _>::from_iter(
        (inventory::iter::<&dyn ToolproofAssertion>)
            .into_iter()
            .map(|i| {
                let segments =
                    parse_segments(i.segments()).expect("builtin assertions should be parseable");

                (segments, *i)
            }),
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_getting_an_instruction() {
        pub struct TestInstruction;

        inventory::submit! {
            &TestInstruction as &dyn ToolproofInstruction
        }

        #[async_trait]
        impl ToolproofInstruction for TestInstruction {
            fn segments(&self) -> &'static str {
                "__test__ I am an instruction asking for {argument}"
            }

            async fn run(
                &self,
                _args: &SegmentArgs<'_>,
                _civ: &mut Civilization,
            ) -> Result<(), ToolproofStepError> {
                Ok(())
            }
        }

        let users_instruction =
            parse_segments("__test__ I am an instruction asking for \"this argument\"")
                .expect("Valid instruction");

        let all_instructions = register_instructions();
        let matching_instruction = all_instructions
            .get(&users_instruction)
            .expect("should be able to retrieve instruction");

        assert_eq!(
            matching_instruction.segments(),
            "__test__ I am an instruction asking for {argument}"
        );
    }

    #[test]
    fn test_getting_a_retriever() {
        pub struct TestRetriever;

        inventory::submit! {
            &TestRetriever as &dyn ToolproofRetriever
        }

        #[async_trait]
        impl ToolproofRetriever for TestRetriever {
            fn segments(&self) -> &'static str {
                "__test__ the file {filename}"
            }

            async fn run(
                &self,
                _args: &SegmentArgs<'_>,
                _civ: &mut Civilization,
            ) -> Result<serde_json::Value, ToolproofStepError> {
                Ok(serde_json::Value::Null)
            }
        }

        let users_segments =
            parse_segments("__test__ the file \"index.html\"").expect("Valid instruction");

        let all_segments = register_retrievers();
        let matching_retriever = all_segments
            .get(&users_segments)
            .expect("should be able to retrieve segments");

        assert_eq!(
            matching_retriever.segments(),
            "__test__ the file {filename}"
        );
    }

    #[test]
    fn test_getting_an_assertion() {
        pub struct TestAssertion;

        inventory::submit! {
            &TestAssertion as &dyn ToolproofAssertion
        }

        #[async_trait]
        impl ToolproofAssertion for TestAssertion {
            fn segments(&self) -> &'static str {
                "__test__ be exactly {value}"
            }

            async fn run(
                &self,
                _base_value: serde_json::Value,
                _args: &SegmentArgs<'_>,
                _civ: &mut Civilization,
            ) -> Result<(), ToolproofStepError> {
                Ok(())
            }
        }

        let users_segments =
            parse_segments("__test__ be exactly {my_json}").expect("Valid instruction");

        let all_segments = register_assertions();
        let matching_assertion = all_segments
            .get(&users_segments)
            .expect("should be able to retrieve segments");

        assert_eq!(matching_assertion.segments(), "__test__ be exactly {value}");
    }
}
