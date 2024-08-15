use std::env;

use crate::parser::ToolproofPlatform;

pub fn normalize_line_endings(s: impl AsRef<str>) -> String {
    s.as_ref().replace("\r\n", "\n")
}

pub fn platform_matches(platforms: &Option<Vec<ToolproofPlatform>>) -> bool {
    let Some(platforms) = platforms else {
        return true;
    };
    if platforms.is_empty() {
        return true;
    }
    match env::consts::OS {
        "linux" => platforms.contains(&ToolproofPlatform::Linux),
        "macos" => platforms.contains(&ToolproofPlatform::Mac),
        "windows" => platforms.contains(&ToolproofPlatform::Windows),
        _ => false,
    }
}
