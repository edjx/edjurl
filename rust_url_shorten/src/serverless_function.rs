use crate::value_parser::{deserialize_value, serialize_value, Value};
use edjx::{error, info, kv, HttpRequest, HttpResponse, StatusCode};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

// "URL Shorten" serverless function.
//
// This function accepts a URL, stores it in the KV store, and returns a short
// string (e.g., 6s4bxc5m) that can be used to retrieve the URL.
//
// Input: HTTP GET Request
//   Query parameters:
//   - 'url'          Long URL to be shortened.
//   - 'alias'        (optional) Select a short string. If no alias is specified,
//                    a unique hash value will be derived from the 'url' and
//                    used instead.
//   HTTP headers:
//   - 'password'     (optional) Password that must be specified in order
//                    to modify an existing short link. To change an existing
//                    link, specify the existing short string as an 'alias'
//                    query parameter, and the previously-set password in
//                    the 'password' header.
//   - 'old_password' (optional) In order to change an existing password,
//                    specify the old password in the 'old_password' header
//                    and the new password in the 'password' header.
//
// Output: HTTP Response
//   On success:
//   - HTTP response with a status 200 and a short string in the body.
//   On failure:
//   - HTTP response with a status other than 2xx and an error message
//     in the body.
//

// Set the hash length, but low values introduce more collisions.
// The actual length may get larger to avoid collisions.
const DEFAULT_SHORTENED_LENGTH: usize = 8;

const ALPHABET_SIZE: u16 = 36;

pub fn serverless(req: HttpRequest) -> HttpResponse {
    info!("URL shortening function started");

    // CORS preflight workaround for web browsers
    if req.method() == &edjx::HttpMethod::OPTIONS {
        info!("Web browser sent an OPTIONS request. Responding with CORS data...");
        return HttpResponse::new()
            // .set_header(
            //     "Access-Control-Allow-Origin".parse().unwrap(),
            //     "*".parse().unwrap()
            // )
            .set_header(
                "Access-Control-Allow-Methods".parse().unwrap(),
                "GET".parse().unwrap(),
            )
            .set_header(
                "Access-Control-Allow-Headers".parse().unwrap(),
                "*".parse().unwrap(),
            )
            .set_status(StatusCode::NO_CONTENT);
    }

    if req.method() != &edjx::HttpMethod::GET {
        return HttpResponse::from("GET method expected")
            .set_status(StatusCode::METHOD_NOT_ALLOWED);
    }

    pub fn as_characters(number: u64, len: usize) -> String {
        let mut digits = number;
        let mut result = vec![];
        loop {
            let d = (digits % (ALPHABET_SIZE as u64)) as u32;
            digits /= ALPHABET_SIZE as u64;
            result.push(std::char::from_digit(d, ALPHABET_SIZE as u32).unwrap());
            if digits == 0 {
                break;
            }
        }
        if result.len() > len {
            result.resize(len, '-');
        }
        result.reverse();
        result.into_iter().collect()
    }

    pub fn key_available(key: &String) -> bool {
        match kv::get(&key) {
            Err(edjx::kv::KVError::NotFound) => true,
            _ => false,
        }
    }

    pub fn authenticate(key: &String, password: &Option<String>) -> bool {
        match kv::get(&key) {
            Ok(val) => {
                let value: Value = match deserialize_value(&val) {
                    Ok(val) => val,
                    Err(_) => {
                        error!("Value in KV store is invalid");
                        return false;
                    }
                };
                if value.password.is_none() {
                    // Not password-protected
                    //true
                    // Disable editing of values that don't have a password
                    false
                } else if password.is_some()
                    && password.as_ref().unwrap() == value.password.as_ref().unwrap()
                {
                    // Passwords match
                    true
                } else {
                    // Passwords don't match
                    false
                }
            }
            _ => false,
        }
    }

    // URL to be shortened ("url" query parameter)
    let url: String = match req.query_param_by_name("url") {
        Some(input_url) => {
            // TODO: Validate the URL
            input_url
        }
        None => {
            error!("No url provided in user request");
            return HttpResponse::from("No url provided in user request")
                .set_status(StatusCode::BAD_REQUEST);
        }
    };

    // A custom or existing short string requested ("alias" query parameter)
    let alias: Option<String> = match req.query_param_by_name("alias") {
        Some(input_alias) => {
            // TODO: Validate the alias
            Some(input_alias)
        }
        None => None,
    };

    // Password must be set to enable editing of existing records ("password" header)
    let password: Option<String> = match req.headers().get("password") {
        Some(val) => {
            // TODO: Validate the password
            Some(std::str::from_utf8(val.as_bytes()).unwrap().to_string())
        }
        None => None,
    };

    // An option to change an existing password ("old_password" header)
    let old_password: Option<String> = match req.headers().get("old_password") {
        Some(val) => {
            // TODO: Validate the password
            Some(std::str::from_utf8(val.as_bytes()).unwrap().to_string())
        }
        None => match &password {
            Some(val) => Some(String::from(val)),
            None => None,
        },
    };

    // A short string that represents the URL.
    // It can be defined explicitly (alias),
    // otherwise it will be generated automatically.
    let shortened: String = match &alias {
        Some(val) => val.to_owned(),
        None => {
            let mut hasher = DefaultHasher::new();
            let mut length = DEFAULT_SHORTENED_LENGTH;
            let mut val;
            let mut iterations = 0;
            loop {
                hasher.write(url.as_bytes());
                let url_hash = hasher.finish();
                val = as_characters(url_hash, length);
                if key_available(&val) {
                    // TODO: Check against a dictionary and skip
                    // strings that resemble actual words
                    break;
                }
                iterations += 1;
                if iterations >= u64::pow(ALPHABET_SIZE as u64, length as u32) {
                    // Add one more character to avoid infinite loops
                    length += 1;
                }
            }
            val
        }
    };

    // If a key doesn't exist, it can be created.
    // If the key exists, the value can be modified only with the correct password.
    if key_available(&shortened) || authenticate(&shortened, &old_password) {
        let val = serialize_value(&Value { url, password });
        match kv::put(&shortened, val, None) {
            Err(e) => HttpResponse::from(format!("{}", e)).set_status(StatusCode::BAD_REQUEST),
            Ok(_) => HttpResponse::from(format!("{}", &shortened)).set_status(StatusCode::OK),
        }
    } else {
        match &alias {
            Some(_) => HttpResponse::from(
                    "Requested alias is already taken and your password doesn't grant you a permission to change it"
                )
                .set_status(StatusCode::FORBIDDEN),
            None => HttpResponse::from(
                    "Generated short string already exists"
                )
                .set_status(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}
