use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use chromiumoxide::cdp::browser_protocol::page::{
    CaptureScreenshotFormat, CaptureScreenshotParams,
};
use chromiumoxide::cdp::browser_protocol::target::{
    CreateBrowserContextParams, CreateTargetParams,
};
use chromiumoxide::cdp::js_protocol::runtime::RemoteObjectType;
use chromiumoxide::error::CdpError;
use chromiumoxide::handler::viewport::Viewport;
use chromiumoxide::page::ScreenshotParams;
use futures::StreamExt;
use tempfile::tempdir;
use tokio::task::JoinHandle;

use crate::civilization::Civilization;
use crate::errors::{
    ToolproofInputError, ToolproofInternalError, ToolproofStepError, ToolproofTestFailure,
};
use crate::options::ToolproofParams;

use super::{SegmentArgs, ToolproofInstruction, ToolproofRetriever};

use chromiumoxide::browser::{Browser, BrowserConfig};
use pagebrowse::{PagebrowseBuilder, Pagebrowser, PagebrowserWindow};

mod browser_specific;

const HARNESS: &'static str = include_str!("./harness.js");
const INIT_SCRIPT: &'static str = include_str!("./init.js");

fn harnessed(js: String) -> String {
    HARNESS.replace("// insert_toolproof_inner_js", &js)
}

fn init_script(timeout_secs: u64) -> String {
    INIT_SCRIPT.replace("DEFAULT_TIMEOUT", &(timeout_secs * 1000).to_string())
}

/// We want selector steps to timeout before the step itself does,
/// since it provides a better error. This makes that more likely.
fn auto_selector_timeout(civ: &Civilization) -> u64 {
    civ.universe.ctx.params.timeout.saturating_sub(2).max(1)
}

fn escape_xpath_string(s: &str) -> String {
    if s.contains('\'') {
        // If string contains single quotes, split on them and wrap with xpath's concat()
        let parts: Vec<_> = s.split('\'').collect();
        format!("concat('{}')", parts.join("',\"'\",'"))
    } else {
        format!("'{}'", s)
    }
}

pub enum BrowserTester {
    Pagebrowse(Arc<Pagebrowser>),
    Chrome {
        browser: Arc<Browser>,
        browser_timeout: u64,
        event_thread: Arc<JoinHandle<Result<(), std::io::Error>>>,
    },
}

async fn try_launch_browser(mut max: usize) -> (Browser, chromiumoxide::Handler) {
    let mut launch = Err(CdpError::NotFound);
    while launch.is_err() && max > 0 {
        max -= 1;
        launch = Browser::launch(
            BrowserConfig::builder()
                .headless_mode(chromiumoxide::browser::HeadlessMode::New)
                .user_data_dir(tempdir().expect("testing on a system with a temp dir"))
                .viewport(Some(Viewport {
                    width: 1600,
                    height: 900,
                    device_scale_factor: Some(2.0),
                    emulating_mobile: false,
                    is_landscape: true,
                    has_touch: false,
                }))
                .build()
                .unwrap(),
        )
        .await;
    }
    match launch {
        Ok(res) => res,
        Err(e) => {
            panic!("Failed to launch browser due to error: {e}");
        }
    }
}

enum InteractionType {
    Click,
    Hover,
}

impl BrowserTester {
    async fn initialize(params: &ToolproofParams) -> Self {
        match params.browser {
            crate::options::ToolproofBrowserImpl::Chrome => {
                let (browser, mut handler) = try_launch_browser(3).await;

                BrowserTester::Chrome {
                    browser: Arc::new(browser),
                    browser_timeout: params.browser_timeout,
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
                    .init_script(init_script(params.browser_timeout))
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
            BrowserTester::Chrome {
                browser,
                browser_timeout,
                ..
            } => {
                let context = browser
                    .create_browser_context(CreateBrowserContextParams {
                        dispose_on_detach: Some(true),
                        proxy_server: None,
                        proxy_bypass_list: None,
                        origins_with_universal_network_access: None,
                    })
                    .await
                    .unwrap();
                let page = browser
                    .new_page(CreateTargetParams {
                        url: "about:blank".to_string(),
                        for_tab: None,
                        width: None,
                        height: None,
                        browser_context_id: Some(context),
                        enable_begin_frame_control: None,
                        new_window: None,
                        background: None,
                    })
                    .await
                    .unwrap();
                page.evaluate_on_new_document(init_script(*browser_timeout))
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

    async fn screenshot_page(&self, filepath: PathBuf) -> Result<(), ToolproofStepError> {
        match self {
            BrowserWindow::Chrome(page) => {
                let image_format = browser_specific::chrome_image_format(&filepath)?;

                page.save_screenshot(
                    ScreenshotParams {
                        cdp_params: CaptureScreenshotParams {
                            format: Some(image_format),
                            ..CaptureScreenshotParams::default()
                        },
                        full_page: Some(false),
                        omit_background: Some(false),
                    },
                    filepath,
                )
                .await
                .map(|_| ())
                .map_err(|e| ToolproofStepError::Internal(e.into()))
            }
            BrowserWindow::Pagebrowse(_) => Err(ToolproofStepError::Internal(
                ToolproofInternalError::Custom {
                    msg: "Screenshots not yet implemented for Pagebrowse".to_string(),
                },
            )),
        }
    }

    async fn screenshot_element(
        &self,
        selector: &str,
        filepath: PathBuf,
        timeout_secs: u64,
    ) -> Result<(), ToolproofStepError> {
        match self {
            BrowserWindow::Chrome(page) => {
                let image_format = browser_specific::chrome_image_format(&filepath)?;

                let element = browser_specific::wait_for_chrome_element_selector(
                    page,
                    selector,
                    timeout_secs,
                )
                .await?;

                element
                    .save_screenshot(image_format, filepath)
                    .await
                    .map(|_| ())
                    .map_err(|e| ToolproofStepError::Internal(e.into()))
            }
            BrowserWindow::Pagebrowse(_) => Err(ToolproofStepError::Internal(
                ToolproofInternalError::Custom {
                    msg: "Screenshots not yet implemented for Pagebrowse".to_string(),
                },
            )),
        }
    }

    async fn interact_text(
        &self,
        text: &str,
        interaction: InteractionType,
        timeout_secs: u64,
    ) -> Result<(), ToolproofStepError> {
        match self {
            BrowserWindow::Chrome(page) => {
                let text = text.to_lowercase();
                let selector_text = escape_xpath_string(&text);
                let el_xpath = |el: &str| {
                    format!("//{el}[contains(translate(., 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', 'abcdefghijklmnopqrstuvwxyz'), {selector_text})]")
                };
                let xpath = [
                    el_xpath("a"),
                    el_xpath("button"),
                    el_xpath("input"),
                    el_xpath("option"),
                    el_xpath("*[@role='button']"),
                    el_xpath("*[@role='option']"),
                ]
                .join(" | ");

                loop {
                    let elements = browser_specific::wait_for_chrome_xpath_selectors(
                        page,
                        &xpath,
                        &format!("with text '{text}'"),
                        timeout_secs,
                    )
                    .await?;

                    if elements.is_empty() {
                        return Err(ToolproofStepError::Assertion(
                            ToolproofTestFailure::Custom {
                                msg: format!(
                                    "Clickable element containing text '{text}' does not exist."
                                ),
                            },
                        ));
                    }

                    if elements.len() > 1 {
                        return Err(ToolproofStepError::Assertion(
                            ToolproofTestFailure::Custom {
                                msg: format!(
                                "Found more than one clickable element containing text '{text}'."
                            ),
                            },
                        ));
                    }

                    if let Err(e) = elements[0].scroll_into_view().await {
                        match e {
                            // If the element was detached from the DOM after the time we selected it,
                            // we want to restart this section and select a new element.
                            CdpError::ScrollingFailed(msg) if msg.contains("detached") => continue,
                            _ => {
                                return Err(ToolproofStepError::Assertion(ToolproofTestFailure::Custom {
                                    msg: format!(
                                        "Element with text '{text}' could not be scrolled into view: {e}"
                                    ),
                                }))
                            }
                        }
                    }

                    let center = match elements[0].clickable_point().await {
                        Ok(c) => c,
                        Err(e) => {
                            if let Ok(res) = elements[0]
                                .call_js_fn("async function() { return this.isConnected; }", true)
                                .await
                            {
                                // If we can't find the center due to the element now being detached from the DOM,
                                // we want to restart this section and select a new element.
                                if matches!(res.result.value, Some(serde_json::Value::Bool(false)))
                                {
                                    continue;
                                }
                            }

                            return Err(ToolproofStepError::Assertion(ToolproofTestFailure::Custom {
                                msg: format!(
                                "Could not find a clickable point for element with text '{text}': {e}"
                            ),
                            }));
                        }
                    };

                    match interaction {
                        InteractionType::Click => {
                            page.click(center).await.map_err(|e| {
                                ToolproofStepError::Assertion(ToolproofTestFailure::Custom {
                                    msg: format!(
                                        "Element with text '{text}' could not be clicked: {e}"
                                    ),
                                })
                            })?;
                        }
                        InteractionType::Hover => {
                            page.move_mouse(center).await.map_err(|e| {
                                ToolproofStepError::Assertion(ToolproofTestFailure::Custom {
                                    msg: format!(
                                        "Element with text '{text}' could not be hovered: {e}"
                                    ),
                                })
                            })?;
                        }
                    }

                    break;
                }

                Ok(())
            }
            BrowserWindow::Pagebrowse(_) => Err(ToolproofStepError::Internal(
                ToolproofInternalError::Custom {
                    msg: "Clicks not yet implemented for Pagebrowse".to_string(),
                },
            )),
        }
    }

    async fn interact_selector(
        &self,
        selector: &str,
        interaction: InteractionType,
        timeout_secs: u64,
    ) -> Result<(), ToolproofStepError> {
        match self {
            BrowserWindow::Chrome(page) => {
                loop {
                    let element = browser_specific::wait_for_chrome_element_selector(
                        page,
                        selector,
                        timeout_secs,
                    )
                    .await?;

                    if let Err(e) = element.scroll_into_view().await {
                        match e {
                            // If the element was detached from the DOM after the time we selected it,
                            // we want to restart this section and select a new element.
                            CdpError::ScrollingFailed(msg) if msg.contains("detached") => continue,
                            _ => {
                                return Err(ToolproofStepError::Assertion(
                                    ToolproofTestFailure::Custom {
                                        msg: format!(
                                        "Element {selector} could not be scrolled into view: {e}"
                                    ),
                                    },
                                ))
                            }
                        }
                    }

                    let center = match element.clickable_point().await {
                        Ok(c) => c,
                        Err(e) => {
                            if let Ok(res) = element
                                .call_js_fn("async function() { return this.isConnected; }", true)
                                .await
                            {
                                // If we can't find the center due to the element now being detached from the DOM,
                                // we want to restart this section and select a new element.
                                if matches!(res.result.value, Some(serde_json::Value::Bool(false)))
                                {
                                    continue;
                                }
                            }

                            return Err(ToolproofStepError::Assertion(
                                ToolproofTestFailure::Custom {
                                    msg: format!(
                                        "Could not find a clickable point for {selector}: {e}"
                                    ),
                                },
                            ));
                        }
                    };

                    match interaction {
                        InteractionType::Click => {
                            page.click(center).await.map_err(|e| {
                                ToolproofStepError::Assertion(ToolproofTestFailure::Custom {
                                    msg: format!("Element {selector} could not be clicked: {e}"),
                                })
                            })?;
                        }
                        InteractionType::Hover => {
                            page.move_mouse(center).await.map_err(|e| {
                                ToolproofStepError::Assertion(ToolproofTestFailure::Custom {
                                    msg: format!("Element {selector} could not be hovered: {e}"),
                                })
                            })?;
                        }
                    }
                    break;
                }

                Ok(())
            }
            BrowserWindow::Pagebrowse(_) => Err(ToolproofStepError::Internal(
                ToolproofInternalError::Custom {
                    msg: "Clicks not yet implemented for Pagebrowse".to_string(),
                },
            )),
        }
    }

    async fn press_key(&self, key: &str, timeout_secs: u64) -> Result<(), ToolproofStepError> {
        match self {
            BrowserWindow::Chrome(page) => {
                let dom =
                    browser_specific::wait_for_chrome_element_selector(page, "body", timeout_secs)
                        .await?;

                dom.press_key(key).await.map_err(|e| {
                    ToolproofStepError::Assertion(ToolproofTestFailure::Custom {
                        msg: format!("Key {key} could not be pressed: {e}"),
                    })
                })?;

                Ok(())
            }
            BrowserWindow::Pagebrowse(_) => Err(ToolproofStepError::Internal(
                ToolproofInternalError::Custom {
                    msg: "Keystrokes not yet implemented for Pagebrowse".to_string(),
                },
            )),
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

pub mod screenshots {
    use crate::errors::{ToolproofInternalError, ToolproofTestFailure};

    use super::*;

    pub struct ScreenshotViewport;

    inventory::submit! {
        &ScreenshotViewport as &dyn ToolproofInstruction
    }

    #[async_trait]
    impl ToolproofInstruction for ScreenshotViewport {
        fn segments(&self) -> &'static str {
            "In my browser, I screenshot the viewport to {filepath}"
        }

        async fn run(
            &self,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<(), ToolproofStepError> {
            let filepath = args.get_string("filepath")?;
            let resolved_path = civ.tmp_file_path(&filepath);
            civ.ensure_path(&resolved_path);

            let Some(window) = civ.window.as_ref() else {
                return Err(ToolproofStepError::External(
                    ToolproofInputError::StepRequirementsNotMet {
                        reason: "no page has been loaded into the browser for this test".into(),
                    },
                ));
            };

            window.screenshot_page(resolved_path).await
        }
    }

    pub struct ScreenshotElement;

    inventory::submit! {
        &ScreenshotElement as &dyn ToolproofInstruction
    }

    #[async_trait]
    impl ToolproofInstruction for ScreenshotElement {
        fn segments(&self) -> &'static str {
            "In my browser, I screenshot the element {selector} to {filepath}"
        }

        async fn run(
            &self,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<(), ToolproofStepError> {
            let selector = args.get_string("selector")?;
            let filepath = args.get_string("filepath")?;
            let resolved_path = civ.tmp_file_path(&filepath);
            civ.ensure_path(&resolved_path);

            let Some(window) = civ.window.as_ref() else {
                return Err(ToolproofStepError::External(
                    ToolproofInputError::StepRequirementsNotMet {
                        reason: "no page has been loaded into the browser for this test".into(),
                    },
                ));
            };

            window
                .screenshot_element(&selector, resolved_path, auto_selector_timeout(civ))
                .await
        }
    }
}

mod interactions {
    use super::*;

    pub struct ClickText;

    inventory::submit! {
        &ClickText as &dyn ToolproofInstruction
    }

    #[async_trait]
    impl ToolproofInstruction for ClickText {
        fn segments(&self) -> &'static str {
            "In my browser, I click {text}"
        }

        async fn run(
            &self,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<(), ToolproofStepError> {
            let text = args.get_string("text")?;

            let Some(window) = civ.window.as_ref() else {
                return Err(ToolproofStepError::External(
                    ToolproofInputError::StepRequirementsNotMet {
                        reason: "no page has been loaded into the browser for this test".into(),
                    },
                ));
            };

            window
                .interact_text(&text, InteractionType::Click, auto_selector_timeout(civ))
                .await
        }
    }

    pub struct HoverText;

    inventory::submit! {
        &HoverText as &dyn ToolproofInstruction
    }

    #[async_trait]
    impl ToolproofInstruction for HoverText {
        fn segments(&self) -> &'static str {
            "In my browser, I hover {text}"
        }

        async fn run(
            &self,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<(), ToolproofStepError> {
            let text = args.get_string("text")?;

            let Some(window) = civ.window.as_ref() else {
                return Err(ToolproofStepError::External(
                    ToolproofInputError::StepRequirementsNotMet {
                        reason: "no page has been loaded into the browser for this test".into(),
                    },
                ));
            };

            window
                .interact_text(&text, InteractionType::Hover, auto_selector_timeout(civ))
                .await
        }
    }

    pub struct ClickSelector;

    inventory::submit! {
        &ClickSelector as &dyn ToolproofInstruction
    }

    #[async_trait]
    impl ToolproofInstruction for ClickSelector {
        fn segments(&self) -> &'static str {
            "In my browser, I click the selector {selector}"
        }

        async fn run(
            &self,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<(), ToolproofStepError> {
            let selector = args.get_string("selector")?;

            let Some(window) = civ.window.as_ref() else {
                return Err(ToolproofStepError::External(
                    ToolproofInputError::StepRequirementsNotMet {
                        reason: "no page has been loaded into the browser for this test".into(),
                    },
                ));
            };

            window
                .interact_selector(
                    &selector,
                    InteractionType::Click,
                    auto_selector_timeout(civ),
                )
                .await
        }
    }

    pub struct HoverSelector;

    inventory::submit! {
        &HoverSelector as &dyn ToolproofInstruction
    }

    #[async_trait]
    impl ToolproofInstruction for HoverSelector {
        fn segments(&self) -> &'static str {
            "In my browser, I hover the selector {selector}"
        }

        async fn run(
            &self,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<(), ToolproofStepError> {
            let selector = args.get_string("selector")?;

            let Some(window) = civ.window.as_ref() else {
                return Err(ToolproofStepError::External(
                    ToolproofInputError::StepRequirementsNotMet {
                        reason: "no page has been loaded into the browser for this test".into(),
                    },
                ));
            };

            window
                .interact_selector(
                    &selector,
                    InteractionType::Hover,
                    auto_selector_timeout(civ),
                )
                .await
        }
    }

    pub struct PressKey;

    inventory::submit! {
        &PressKey as &dyn ToolproofInstruction
    }

    #[async_trait]
    impl ToolproofInstruction for PressKey {
        fn segments(&self) -> &'static str {
            "In my browser, I press the {keyname} key"
        }

        async fn run(
            &self,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<(), ToolproofStepError> {
            let keyname = args.get_string("keyname")?;

            let Some(window) = civ.window.as_ref() else {
                return Err(ToolproofStepError::External(
                    ToolproofInputError::StepRequirementsNotMet {
                        reason: "no page has been loaded into the browser for this test".into(),
                    },
                ));
            };

            window.press_key(&keyname, auto_selector_timeout(civ)).await
        }
    }

    pub struct TypeText;

    inventory::submit! {
        &TypeText as &dyn ToolproofInstruction
    }

    #[async_trait]
    impl ToolproofInstruction for TypeText {
        fn segments(&self) -> &'static str {
            "In my browser, I type {text}"
        }

        async fn run(
            &self,
            args: &SegmentArgs<'_>,
            civ: &mut Civilization,
        ) -> Result<(), ToolproofStepError> {
            let text = args.get_string("text")?;

            let Some(window) = civ.window.as_ref() else {
                return Err(ToolproofStepError::External(
                    ToolproofInputError::StepRequirementsNotMet {
                        reason: "no page has been loaded into the browser for this test".into(),
                    },
                ));
            };

            for c in text.split("").filter(|s| !s.is_empty()) {
                let resolved_key = match c {
                    "\n" => "Enter",
                    "\t" => "Tab",
                    c => c,
                };
                window
                    .press_key(resolved_key, auto_selector_timeout(civ))
                    .await?;
            }

            Ok(())
        }
    }
}
