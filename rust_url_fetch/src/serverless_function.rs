use crate::value_parser::{deserialize_value, Value};
use edjx::{error, info, kv, kv::KVError, HeaderValue, HttpRequest, HttpResponse, StatusCode};

// "URL Fetch" serverless function.
//
// This function accepts a short string (e.g., 6s4bxc5m) that was generated
// by the "URL Shorten" function, retrieves the corresponding stored URL
// from the KV store, and sends a response with a redirect to the stored URL.
//
// Input: HTTP GET Request
//   Query parameters:
//   - 's'       Short string or alias returned by the URL Shorten function
//
// Output: HTTP Response
//   On success:
//   - HTTP response with an HTTP 302 redirect to the stored URL
//   On failure:
//   - HTTP response with a 4xx or 5xx HTTP status code and an error message
//     in the body.
//

pub fn serverless(req: HttpRequest) -> HttpResponse {
    info!("URL decoding function started");

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

    // Short string that represents a stored URL ("s" query parameter)
    let shortstr: String = match req.query_param_by_name("s") {
        Some(val) => val,
        None => {
            error!("No key provided in user request");
            return HttpResponse::from("No key provided in user request")
                .set_status(StatusCode::BAD_REQUEST);
        }
    };

    // Retrieve the original URL from the KV store
    let record = match kv::get(&shortstr) {
        Ok(val) => val,
        Err(e) => {
            let status = match e {
                KVError::Unknown => StatusCode::BAD_REQUEST,
                KVError::UnAuthorized => StatusCode::UNAUTHORIZED,
                KVError::NotFound => StatusCode::NOT_FOUND,
            };
            return HttpResponse::from(e.to_string()).set_status(status);
        }
    };

    // Decode the KV entry
    let value: Value = match deserialize_value(&record) {
        Ok(val) => val,
        Err(_) => {
            error!("Stored record is invalid");
            return HttpResponse::from("Stored record is invalid")
                .set_status(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get the stored URL
    let url = value.url;

    HttpResponse::from(String::from(&url))
        .set_status(StatusCode::FOUND)
        .set_header(
            "Location".parse().unwrap(),
            HeaderValue::from_str(&url).unwrap(),
        )
}
