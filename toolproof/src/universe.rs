use pagebrowse::Pagebrowser;
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    sync::Arc,
};

use crate::{
    definitions::{ToolproofAssertion, ToolproofInstruction, ToolproofRetriever},
    options::ToolproofContext,
    segments::ToolproofSegments,
    ToolproofTestFile,
};

pub struct Universe<'u> {
    pub pagebrowser: Arc<Pagebrowser>,
    pub tests: BTreeMap<String, ToolproofTestFile>,
    pub instructions: HashMap<ToolproofSegments, &'u dyn ToolproofInstruction>,
    pub instruction_comparisons: Vec<String>,
    pub retrievers: HashMap<ToolproofSegments, &'u dyn ToolproofRetriever>,
    pub retriever_comparisons: Vec<String>,
    pub assertions: HashMap<ToolproofSegments, &'u dyn ToolproofAssertion>,
    pub assertion_comparisons: Vec<String>,
    pub ctx: ToolproofContext,
}
