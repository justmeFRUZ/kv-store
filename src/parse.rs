//! Hand-written HTTP/1.1 request parser.
//!
//! Pure function: takes bytes, returns a value. No I/O here.

#[derive(Debug)]
pub enum ParseResult {
    Incomplete,
    Complete { request: Request, consumed: usize },
    Error,
}

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

pub fn parse_request(input: &[u8]) -> ParseResult {
    fn find_crlf(input: &[u8]) -> Option<usize> {
        (0..input.len().saturating_sub(1)).find(|&i| input[i] == b'\r' && input[i + 1] == b'\n')
    }

    fn find_crlfcrlf(input: &[u8]) -> Option<usize> {
        (0..input.len().saturating_sub(3)).find(|&i| {
            input[i] == b'\r'
                && input[i + 1] == b'\n'
                && input[i + 2] == b'\r'
                && input[i + 3] == b'\n'
        })
    }

    fn has_whitespace(input: &[u8]) -> bool {
        input.iter().any(|b| b.is_ascii_whitespace())
    }

    fn has_bare_lf(input: &[u8]) -> bool {
        input
            .iter()
            .enumerate()
            .any(|(i, &b)| b == b'\n' && (i == 0 || input[i - 1] != b'\r'))
    }

    fn trim_ascii_whitespace(input: &[u8]) -> &[u8] {
        let start = input
            .iter()
            .position(|b| !b.is_ascii_whitespace())
            .unwrap_or(input.len());
        let end = input
            .iter()
            .rposition(|b| !b.is_ascii_whitespace())
            .map(|i| i + 1)
            .unwrap_or(0);
        if start > end {
            &input[0..0]
        } else {
            &input[start..end]
        }
    }

    if has_bare_lf(input) {
        return ParseResult::Error;
    }

    let line_end = match find_crlf(input) {
        Some(i) => i,
        None => return ParseResult::Incomplete,
    };

    let line = &input[..line_end];
    let parts: Vec<&[u8]> = line.split(|&b| b == b' ').collect();
    if parts.len() != 3 || parts.iter().any(|p| p.is_empty()) {
        return ParseResult::Error;
    }

    let method = match std::str::from_utf8(parts[0]) {
        Ok(s) => s.to_string(),
        Err(_) => return ParseResult::Error,
    };

    let path = match std::str::from_utf8(parts[1]) {
        Ok(s) => s.to_string(),
        Err(_) => return ParseResult::Error,
    };
    let version = match std::str::from_utf8(parts[2]) {
        Ok(s) => s.to_string(),
        Err(_) => return ParseResult::Error,
    };

    let header_end = match find_crlfcrlf(input) {
        Some(j) => j,
        None => return ParseResult::Incomplete,
    };

    let mut headers = Vec::new();
    let mut pos = line_end + 2;
    while pos < header_end {
        let next = match find_crlf(&input[pos..]) {
            Some(i) => pos + i,
            None => return ParseResult::Error,
        };
        let header_line = &input[pos..next];
        let colon = match header_line.iter().position(|&b| b == b':') {
            Some(i) => i,
            None => return ParseResult::Error,
        };
        let name_bytes = &header_line[..colon];
        let value_bytes = &header_line[colon + 1..];
        if name_bytes.is_empty() || has_whitespace(name_bytes) {
            return ParseResult::Error;
        }
        let name = match std::str::from_utf8(name_bytes) {
            Ok(s) => s.to_ascii_lowercase(),
            Err(_) => return ParseResult::Error,
        };
        let value = match std::str::from_utf8(trim_ascii_whitespace(value_bytes)) {
            Ok(s) => s.to_string(),
            Err(_) => return ParseResult::Error,
        };
        headers.push((name, value));
        pos = next + 2;
        // Parse header
    }
    ParseResult::Complete {
        request: Request {
            method,
            path,
            version,
            headers,
            body: Vec::new(),
        },
        consumed: header_end + 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type HeaderCase = (&'static [u8], &'static [(&'static str, &'static str)]);
    #[test]
    fn request_line_valid() {
        let cases: &[(&[u8], &str, &str, &str)] = &[
            (b"GET / HTTP/1.1\r\n\r\n" as &[u8], "GET", "/", "HTTP/1.1"),
            (
                b"PUT /kv/foo HTTP/1.1\r\n\r\n" as &[u8],
                "PUT",
                "/kv/foo",
                "HTTP/1.1",
            ),
            (
                b"GET /range?from=a&to=z HTTP/1.1\r\n\r\n" as &[u8],
                "GET",
                "/range?from=a&to=z",
                "HTTP/1.1",
            ),
        ];
        for (input, method, path, version) in cases {
            match parse_request(input) {
                ParseResult::Complete { request, consumed } => {
                    assert_eq!(&request.method, method, "method for {:?}", input);
                    assert_eq!(&request.path, path, "path for {:?}", input);
                    assert_eq!(&request.version, version, "version for {:?}", input);
                    assert_eq!(consumed, input.len(), "consumed for {:?}", input);
                }
                other => panic!("expected Complete for {:?}, got {:?}", input, other),
            }
        }
    }

    #[test]
    fn request_line_incomplete() {
        let cases: &[&[u8]] = &[
            b"" as &[u8],
            b"GET" as &[u8],
            b"GET / HTT" as &[u8],
            b"GET / HTTP/1.1\r\n" as &[u8],
        ];
        for input in cases {
            assert!(
                matches!(parse_request(input), ParseResult::Incomplete),
                "expected Incomplete for {:?}",
                input
            );
        }
    }

    #[test]
    fn request_line_error() {
        let cases: &[&[u8]] = &[
            b"GET\r\n\r\n" as &[u8],
            b"GET /\r\n\r\n" as &[u8],
            b"GET / HTTP/1.1 EXTRA\r\n\r\n" as &[u8],
            b"GET  / HTTP/1.1\r\n\r\n" as &[u8],
        ];
        for input in cases {
            assert!(
                matches!(parse_request(input), ParseResult::Error),
                "expected Error for {:?}",
                input
            );
        }
    }
    #[test]
    fn headers_valid() {
        let cases: &[HeaderCase] = &[
            (
                b"GET / HTTP/1.1\r\nHost: example\r\n\r\n" as &[u8],
                &[("host", "example")] as &[(&str, &str)],
            ),
            (
                b"GET / HTTP/1.1\r\nhost: example\r\n\r\n" as &[u8],
                &[("host", "example")] as &[(&str, &str)],
            ),
            (
                b"GET / HTTP/1.1\r\nContent-Type: text/plain\r\nX-Foo: hello world\r\n\r\n"
                    as &[u8],
                &[("content-type", "text/plain"), ("x-foo", "hello world")] as &[(&str, &str)],
            ),
        ];
        for (input, expected) in cases {
            match parse_request(input) {
                ParseResult::Complete { request, .. } => {
                    let got: Vec<(&str, &str)> = request
                        .headers
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str()))
                        .collect();
                    assert_eq!(&got, expected, "headers for {:?}", input);
                }
                other => panic!("expected Complete for {:?}, got {:?}", input, other),
            }
        }
    }

    #[test]
    fn headers_error() {
        let cases: &[&[u8]] = &[
            b"GET / HTTP/1.1\r\nHost : example\r\n\r\n" as &[u8],
            b"GET / HTTP/1.1\r\nHost example\r\n\r\n" as &[u8],
            b"GET / HTTP/1.1\r\n: example\r\n\r\n" as &[u8],
        ];
        for input in cases {
            assert!(
                matches!(parse_request(input), ParseResult::Error),
                "expected Error for {:?}",
                input
            );
        }
    }
}
