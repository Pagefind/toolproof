// Much of this file's code is derived from [insta](https://github.com/mitsuhiko/insta)
// The original code is licensed under the Apache License, Version 2.0:
//
// Copyright 2020 Armin Ronacher
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{borrow::Cow, time::Duration};

use console::style;
use similar::{Algorithm, ChangeTag, TextDiff};

pub fn diff_snapshots(old: &str, new: &str) -> String {
    let newlines_matter = newlines_matter(old, new);
    let diff = TextDiff::configure()
        .algorithm(Algorithm::Patience)
        .timeout(Duration::from_millis(500))
        .diff_lines(old, new);

    let mut lines = vec![];

    for op in diff.ops() {
        for change in diff.iter_inline_changes(op) {
            match change.tag() {
                ChangeTag::Insert => {
                    let mut s = format!(
                        "{:>5} {:>5} │{}",
                        "",
                        style(change.new_index().unwrap()).cyan().dim().bold(),
                        style("+").green(),
                    );

                    let has_emphasis = change.values().iter().any(|(e, _)| *e);

                    for &(emphasized, change) in change.values() {
                        let change = render_invisible(change, newlines_matter);
                        if !has_emphasis {
                            s.push_str(&format!("{}", style(change).green()));
                        } else if emphasized {
                            s.push_str(&format!("{}", style(change).green().underlined()));
                        } else {
                            s.push_str(&format!("{}", style(change).green().dim()));
                        }
                    }
                    lines.push(s);
                }
                ChangeTag::Delete => {
                    let mut s = format!(
                        "{:>5} {:>5} │{}",
                        style(change.old_index().unwrap()).cyan().dim(),
                        "",
                        style("-").red(),
                    );
                    for &(emphasized, change) in change.values() {
                        let change = render_invisible(change, newlines_matter);
                        if emphasized {
                            s.push_str(&format!("{}", style(change).red().underlined()));
                        } else {
                            s.push_str(&format!("{}", style(change).red().dim()));
                        }
                    }
                    lines.push(s);
                }
                ChangeTag::Equal => {
                    let mut s = format!(
                        "{:>5} {:>5} │ ",
                        style(change.old_index().unwrap()).cyan().dim(),
                        style(change.new_index().unwrap()).cyan().dim().bold(),
                    );
                    for &(_, change) in change.values() {
                        let change = render_invisible(change, newlines_matter);
                        s.push_str(&format!("{}", style(change).dim()));
                    }
                    lines.push(s);
                }
            }

            if change.missing_newline() {
                lines.push("\n".to_string());
            }
        }
    }

    lines.join("")
}

fn trailing_newline(s: &str) -> &str {
    if s.ends_with("\r\n") {
        "\r\n"
    } else if s.ends_with('\r') {
        "\r"
    } else if s.ends_with('\n') {
        "\n"
    } else {
        ""
    }
}

fn detect_newlines(s: &str) -> (bool, bool, bool) {
    let mut last_char = None;
    let mut detected_crlf = false;
    let mut detected_cr = false;
    let mut detected_lf = false;

    for c in s.chars() {
        if c == '\n' {
            if last_char.take() == Some('\r') {
                detected_crlf = true;
            } else {
                detected_lf = true;
            }
        }
        if last_char == Some('\r') {
            detected_cr = true;
        }
        last_char = Some(c);
    }
    if last_char == Some('\r') {
        detected_cr = true;
    }

    (detected_cr, detected_crlf, detected_lf)
}

fn newlines_matter(left: &str, right: &str) -> bool {
    if trailing_newline(left) != trailing_newline(right) {
        return true;
    }

    let (cr1, crlf1, lf1) = detect_newlines(left);
    let (cr2, crlf2, lf2) = detect_newlines(right);

    !matches!(
        (cr1 || cr2, crlf1 || crlf2, lf1 || lf2),
        (false, false, false) | (true, false, false) | (false, true, false) | (false, false, true)
    )
}

fn render_invisible(s: &str, newlines_matter: bool) -> Cow<'_, str> {
    if newlines_matter || s.find(&['\x1b', '\x07', '\x08', '\x7f'][..]).is_some() {
        Cow::Owned(
            s.replace('\r', "␍\r")
                .replace('\n', "␊\n")
                .replace("␍\r␊\n", "␍␊\r\n"),
        )
    } else {
        Cow::Borrowed(s)
    }
}
