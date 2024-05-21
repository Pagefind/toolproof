pub fn normalize_line_endings(s: impl AsRef<str>) -> String {
    s.as_ref().replace("\r\n", "\n")
}
