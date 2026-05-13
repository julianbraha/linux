// SPDX-License-Identifier: GPL-2.0-only
use crate::curl_ffi::head_request;
use std::collections::HashSet;

#[derive(PartialEq, Debug)]
pub enum LinkStatus {
    Ok,
    ProbablyBlocked,
    Redirected(String),
    NotFound,
    ServerError,
    Unreachable(String),
    UnsupportedScheme(String),
}

pub fn check_link(url: &str) -> LinkStatus {
    if let Some(scheme) = url.split("://").next() {
        match scheme {
            "http" | "https" => return check_http(url),

            "git" | "ftp" => {
                return LinkStatus::UnsupportedScheme(scheme.into());
            }

            _ => {
                return LinkStatus::UnsupportedScheme(scheme.into());
            }
        }
    }

    LinkStatus::Unreachable("invalid URL".into())
}

fn check_http(url: &str) -> LinkStatus {
    let response = match head_request(url) {
        Ok(r) => r,
        Err(e) => return LinkStatus::Unreachable(e),
    };

    match response.response_code {
        200..=299 => LinkStatus::Ok,

        301 | 302 => LinkStatus::Redirected(response.location.unwrap_or_else(|| "unknown".into())),

        403 | 429 => LinkStatus::ProbablyBlocked,

        404 => LinkStatus::NotFound,

        500..=599 => LinkStatus::ServerError,

        _ => LinkStatus::ProbablyBlocked,
    }
}

pub fn find_links(text: &str) -> Vec<String> {
    fn is_scheme_char(c: u8) -> bool {
        c.is_ascii_alphanumeric() || matches!(c, b'+' | b'-' | b'.')
    }

    fn is_url_terminator(c: u8) -> bool {
        c.is_ascii_whitespace()
            || matches!(
                c,
                b'"' | b'\'' | b'<' | b'>' | b'(' | b')' | b'[' | b']' | b'{' | b'}'
            )
    }

    let bytes = text.as_bytes();

    let mut links = Vec::new();
    let mut seen = HashSet::new();

    let mut i = 0;

    while i + 3 < bytes.len() {
        if bytes[i] == b':' && bytes[i + 1] == b'/' && bytes[i + 2] == b'/' {
            // walk backward to find scheme start
            let mut start = i;

            while start > 0 && is_scheme_char(bytes[start - 1]) {
                start -= 1;
            }

            // require non-empty scheme
            if start == i {
                i += 3;
                continue;
            }

            // first char must be alphabetic
            if !bytes[start].is_ascii_alphabetic() {
                i += 3;
                continue;
            }

            // walk forward to url end
            let mut end = i + 3;

            while end < bytes.len() && !is_url_terminator(bytes[end]) {
                end += 1;
            }

            let mut url = &text[start..end];

            // trim trailing punctuation
            url = url.trim_end_matches(&['.', ',', ';', ':', '!', '?'][..]);

            // trim unmatched markdown
            while let Some(last) = url.chars().last() {
                let trim = match last {
                    ')' => url.matches('(').count() < url.matches(')').count(),

                    ']' => url.matches('[').count() < url.matches(']').count(),

                    '}' => url.matches('{').count() < url.matches('}').count(),

                    _ => false,
                };

                if trim {
                    url = &url[..url.len() - last.len_utf8()];
                } else {
                    break;
                }
            }

            if seen.insert(url) {
                links.push(url.to_string());
            }

            i = end;
        } else {
            i += 1;
        }
    }

    links
}
