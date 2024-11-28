use std::collections::{BTreeMap, HashMap};

use tokio::sync::OnceCell;

use crate::{
    definitions::{
        browser::BrowserTester, ToolproofAssertion, ToolproofInstruction, ToolproofRetriever,
    },
    options::ToolproofContext,
    segments::ToolproofSegments,
    ToolproofMacroFile, ToolproofTestFile,
};

pub struct Universe<'u> {
    pub browser: OnceCell<BrowserTester>,
    pub tests: BTreeMap<String, ToolproofTestFile>,
    pub macros: HashMap<ToolproofSegments, ToolproofMacroFile>,
    pub macro_comparisons: Vec<String>,
    pub instructions: HashMap<ToolproofSegments, &'u dyn ToolproofInstruction>,
    pub instruction_comparisons: Vec<String>,
    pub retrievers: HashMap<ToolproofSegments, &'u dyn ToolproofRetriever>,
    pub retriever_comparisons: Vec<String>,
    pub assertions: HashMap<ToolproofSegments, &'u dyn ToolproofAssertion>,
    pub assertion_comparisons: Vec<String>,
    pub ctx: ToolproofContext,
}
