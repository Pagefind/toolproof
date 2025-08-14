use std::path::PathBuf;

use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;

use crate::errors::{ToolproofInputError, ToolproofStepError, ToolproofTestFailure};

pub fn chrome_image_format(
    filepath: &PathBuf,
) -> Result<CaptureScreenshotFormat, ToolproofStepError> {
    match filepath.extension() {
        Some(ext) => {
            let ext = ext.to_string_lossy().to_lowercase();
            match ext.as_str() {
                "png" => Ok(CaptureScreenshotFormat::Png),
                "webp" => Ok(CaptureScreenshotFormat::Webp),
                "jpg" | "jpeg" => Ok(CaptureScreenshotFormat::Jpeg),
                _ => Err(ToolproofStepError::External(
                    ToolproofInputError::StepRequirementsNotMet {
                        reason: "Image file extension must be png, webp, jpeg, or jpg".to_string(),
                    },
                )),
            }
        }
        None => Err(ToolproofStepError::External(
            ToolproofInputError::StepRequirementsNotMet {
                reason: "Image file path must have an extension".to_string(),
            },
        )),
    }
}

pub async fn wait_for_chrome_element_selector(
    page: &chromiumoxide::Page,
    selector: &str,
    timeout_secs: u64,
) -> Result<chromiumoxide::element::Element, ToolproofStepError> {
    let start = std::time::Instant::now();
    while start.elapsed().as_secs() < timeout_secs {
        if let Ok(element) = page.find_element(selector).await {
            return Ok(element);
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    Err(ToolproofStepError::Assertion(
        ToolproofTestFailure::Custom {
            msg: format!("Element {selector} could not be found within {timeout_secs}s"),
        },
    ))
}

pub async fn wait_for_chrome_xpath_selectors(
    page: &chromiumoxide::Page,
    xpath: &str,
    descriptor: &str,
    timeout_secs: u64,
) -> Result<Vec<chromiumoxide::element::Element>, ToolproofStepError> {
    let start = std::time::Instant::now();
    while start.elapsed().as_secs() < timeout_secs {
        if let Ok(elements) = page.find_xpaths(xpath).await {
            if !elements.is_empty() {
                return Ok(elements);
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    Err(ToolproofStepError::Assertion(
        ToolproofTestFailure::Custom {
            msg: format!("Element {descriptor} could not be found within {timeout_secs}s"),
        },
    ))
}
