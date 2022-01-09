use std::collections::HashMap;

use my_http_utils::{HttpFailResult, HttpOkResult, WebContentType};
use tokio::{fs::File, io::AsyncReadExt};

pub const DEFAULT_FOLDER: &str = "./wwwroot";

pub async fn serve_file_with_placeholders(
    filename: &str,
    content_type: Option<WebContentType>,
    placeholders: &HashMap<&str, String>,
) -> Result<HttpOkResult, HttpFailResult> {
    // Serve a file by asynchronously reading it by chunks using tokio-util crate.

    match get(filename).await {
        Ok(content) => {
            let content = replace_placeholders(content.as_slice(), placeholders);
            let result = HttpOkResult::Content {
                content_type,
                content,
            };

            Ok(result)
        }
        Err(err) => {
            let err = format!("Error handing file: {:?}. Filename: {}", err, filename);
            Err(HttpFailResult::as_not_found(err, false))
        }
    }
}

pub fn replace_placeholders(src: &[u8], placeholders: &HashMap<&str, String>) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();

    let mut i = 0;
    while i < src.len() {
        if src[i] == b'*' && src[i + 1] == b'*' && src[i + 2] == b'*' {
            let end_index = find_end_of_placeholder(src, i + 3).unwrap();

            let key = std::str::from_utf8(&src[i + 3..end_index]).unwrap();

            let value = placeholders.get(key);

            if let Some(value) = value {
                result.extend(value.as_bytes());
            }
            i = end_index + 2;
        } else {
            result.push(src[i]);
        }

        i += 1;
    }

    result
}

pub fn find_end_of_placeholder(src: &[u8], placeholder_start: usize) -> Option<usize> {
    for i in placeholder_start..src.len() {
        if src[i] == b'*' {
            return Some(i);
        }
    }

    None
}

pub async fn get(filename: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(filename).await?;

    let mut result: Vec<u8> = Vec::new();

    loop {
        let res = file.read_buf(&mut result).await?;

        if res == 0 {
            break;
        }
    }

    return Ok(result);
}
