use std::sync::Arc;

use async_trait::async_trait;
use chromiumoxide::cdp::browser_protocol::target::CreateTargetParams;
use futures::StreamExt;
use tokio::task::JoinHandle;

use crate::civilization::Civilization;
use crate::errors::{ToolproofInputError, ToolproofStepError};
use crate::options::ToolproofParams;

use super::{SegmentArgs, ToolproofInstruction, ToolproofRetriever};

use chromiumoxide::browser::{Browser, BrowserConfig};
use pagebrowse::{PagebrowseBuilder, Pagebrowser, PagebrowserWindow};

const HARNESS: &'static str = include_str!("./harness.js");
const INIT_SCRIPT: &'static str = include_str!("./init.js");

fn harnessed(js: String) -> String {
    HARNESS.replace("// insert_toolproof_inner_js", &js)
}

pub enum BrowserTester {
    Pagebrowse(Arc<Pagebrowser>),
    Chrome {
        browser: Arc<Browser>,
        event_thread: Arc<JoinHandle<Result<(), std::io::Error>>>,
    },
}

impl BrowserTester {
    async fn initialize(params: &ToolproofParams) -> Self {
        match params.browser {
            crate::options::ToolproofBrowserImpl::Chrome => {
                let (browser, mut handler) =
                    Browser::launch(BrowserConfig::builder().build().unwrap())
                        .await
                        .unwrap();

                BrowserTester::Chrome {
                    browser: Arc::new(browser),
                    event_thread: Arc::new(tokio::task::spawn(async move {
                        loop {
                            let _ = handler.next().await.unwrap();
                        }
                    })),
                }
            }
            crate::options::ToolproofBrowserImpl::Pagebrowse => {
                let pagebrowser = PagebrowseBuilder::new(params.concurrency)
                    .visible(false)
                    .manager_path(format!(
                        "{}/../bin/pagebrowse_manager",
                        env!("CARGO_MANIFEST_DIR")
                    ))
                    .init_script(INIT_SCRIPT.to_string())
                    .build()
                    .await
                    .expect("Can't build the pagebrowser");

                BrowserTester::Pagebrowse(Arc::new(pagebrowser))
            }
        }
    }

    async fn get_window(&self) -> BrowserWindow {
        match self {
            BrowserTester::Pagebrowse(pb) => {
                BrowserWindow::Pagebrowse(pb.get_window().await.unwrap())
            }
            BrowserTester::Chrome { browser, .. } => {
                let page = browser
                    .new_page(CreateTargetParams {
                        url: "about:blank".to_string(),
                        width: None,
                        height: None,
                        browser_context_id: None,
                        enable_begin_frame_control: None,
                        new_window: None,
                        background: None,
                    })
                    .await
                    .unwrap();
                page.evaluate_on_new_document(INIT_SCRIPT.to_string())
                    .await
                    .expect("Could not set initialization js");
                BrowserWindow::Chrome(page)
            }
        }
    }
}

pub enum BrowserWindow {
    Chrome(chromiumoxide::Page),
    Pagebrowse(PagebrowserWindow),
}

impl BrowserWindow {
    async fn navigate(&self, url: String, wait_for_load: bool) -> Result<(), ToolproofStepError> {
        match self {
            BrowserWindow::Chrome(page) => {
                // TODO: This is implicitly always wait_for_load: true
                page.goto(url)
                    .await
                    .map(|_| ())
                    .map_err(|inner| ToolproofStepError::Internal(inner.into()))
            }
            BrowserWindow::Pagebrowse(window) => window
                .navigate(url, wait_for_load)
                .await
                .map_err(|inner| ToolproofStepError::Internal(inner.into())),
        }
    }

    async fn evaluate_script(
        &self,
        script: String,
    ) -> Result<Option<serde_json::Value>, ToolproofStepError> {
        match self {
            BrowserWindow::Chrome(page) => {
                let res = page
                    .evaluate_function(format!("async function() {{{}}}", harnessed(script)))
                    .await
                    .map_err(|inner| ToolproofStepError::Internal(inner.into()))?;

                Ok(res.object().value.clone())
            }
            BrowserWindow::Pagebrowse(window) => window
                .evaluate_script(harnessed(script))
                .await
                .map_err(|inner| ToolproofStepError::Internal(inner.into())),
        }
    }
}

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

            let browser = civ
                .universe
                .browser
                .get_or_init(|| async { BrowserTester::initialize(&civ.universe.ctx.params).await })
                .await;

            let window = browser.get_window().await;

            window.navigate(url.to_string(), true).await?;

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

        let value = window.evaluate_script(js).await?;

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
