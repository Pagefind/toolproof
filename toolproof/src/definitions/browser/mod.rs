use std::collections::HashMap;

use async_trait::async_trait;

use crate::civilization::Civilization;
use crate::errors::{ToolproofInputError, ToolproofStepError};

use super::{SegmentArgs, ToolproofInstruction, ToolproofRetriever};

use pagebrowse::{PagebrowseBuilder, Pagebrowser, PagebrowserWindow};

const HARNESS: &'static str = include_str!("./harness.js");

mod load_page {
    use super::*;

    pub struct LoadPage;

    inventory::submit! {
        &LoadPage as &dyn ToolproofInstruction
    }

    #[async_trait]
    impl ToolproofInstruction for LoadPage {
        fn segments(&self) -> &'static str {
            "In my browser, I load {url}"
        }

        async fn run(
            &self,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<(), ToolproofStepError> {
            let url = format!(
                "http://localhost:{}{}",
                civ.ensure_port(),
                args.get_string("url")?
            );

            let window = civ.universe.pagebrowser.get_window().await.unwrap();

            window
                .navigate(url.to_string(), true)
                .await
                .map_err(|inner| ToolproofStepError::Internal(inner.into()))?;

            civ.window = Some(window);

            Ok(())
        }
    }
}

mod eval_js {
    use std::time::Duration;

    use futures::TryFutureExt;
    use tokio::time::sleep;

    use crate::errors::{ToolproofInternalError, ToolproofTestFailure};

    use super::*;

    fn harnessed(js: String) -> String {
        HARNESS.replace("// insert_toolproof_inner_js", &js)
    }

    async fn eval_and_return_js(
        js: String,
        civ: &mut Civilization<'_>,
    ) -> Result<serde_json::Value, ToolproofStepError> {
        let Some(window) = civ.window.as_ref() else {
            return Err(ToolproofStepError::External(
                ToolproofInputError::StepRequirementsNotMet {
                    reason: "no page has been loaded into the browser for this test".into(),
                },
            ));
        };

        let value = window
            .evaluate_script(harnessed(js))
            .await
            .map_err(|inner| ToolproofStepError::Internal(inner.into()))?;

        let Some(serde_json::Value::Object(map)) = &value else {
            return Err(ToolproofStepError::External(
                ToolproofInputError::StepError {
                    reason: "JavaScript failed to parse and run".to_string(),
                },
            ));
        };

        let Some(serde_json::Value::Array(errors)) = map.get("toolproof_errs") else {
            return Err(ToolproofStepError::Internal(
                ToolproofInternalError::Custom {
                    msg: format!("JavaScript returned an unexpected value: {value:?}"),
                },
            ));
        };

        if !errors.is_empty() {
            return Err(ToolproofStepError::Assertion(
                ToolproofTestFailure::BrowserJavascriptErr {
                    msg: errors
                        .iter()
                        .map(|v| v.as_str().unwrap())
                        .collect::<Vec<_>>()
                        .join("\n"),
                    logs: map.get("logs").unwrap().as_str().unwrap().to_string(),
                },
            ));
        }

        Ok(map
            .get("inner_response")
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    }

    pub struct EvalJs;

    inventory::submit! {
        &EvalJs as &dyn ToolproofInstruction
    }

    #[async_trait]
    impl ToolproofInstruction for EvalJs {
        fn segments(&self) -> &'static str {
            "In my browser, I evaluate {js}"
        }

        async fn run(
            &self,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<(), ToolproofStepError> {
            let js = args.get_string("js")?;

            _ = eval_and_return_js(js, civ).await?;

            Ok(())
        }
    }

    pub struct GetJs;

    inventory::submit! {
        &GetJs as &dyn ToolproofRetriever
    }

    #[async_trait]
    impl ToolproofRetriever for GetJs {
        fn segments(&self) -> &'static str {
            "In my browser, the result of {js}"
        }

        async fn run(
            &self,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<serde_json::Value, ToolproofStepError> {
            let js = args.get_string("js")?;

            eval_and_return_js(js, civ).await
        }
    }

    pub struct GetConsole;

    inventory::submit! {
        &GetConsole as &dyn ToolproofRetriever
    }

    #[async_trait]
    impl ToolproofRetriever for GetConsole {
        fn segments(&self) -> &'static str {
            "In my browser, the console"
        }

        async fn run(
            &self,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<serde_json::Value, ToolproofStepError> {
            eval_and_return_js("return toolproof_log_events[`ALL`];".to_string(), civ).await
        }
    }
}
