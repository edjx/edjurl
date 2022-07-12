use edjx::{error, info, kv, kv::KVError, HttpRequest, HttpResponse, StatusCode, HeaderValue};
use crate::value_parser::{Value, deserialize_value};

pub fn serverless(req: HttpRequest) -> HttpResponse {
    info!("URL decoding function started");

    // CORS preflight workaround for web browsers
    if req.method() == &edjx::HttpMethod::OPTIONS {
        info!("Web browser sent an OPTIONS request. Responding with CORS data...");
        return HttpResponse::new()
            //.set_header("Access-Control-Allow-Origin".parse().unwrap(), "*".parse().unwrap())
            .set_header("Access-Control-Allow-Methods".parse().unwrap(), "GET".parse().unwrap())
            .set_header("Access-Control-Allow-Headers".parse().unwrap(), "*".parse().unwrap())
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

    let record = match kv::get(&shortstr) {
        Ok(val) => val,
        Err(e) => {
            let status = match e {
                KVError::Unknown => StatusCode::BAD_REQUEST,
                KVError::UnAuthorized => StatusCode::UNAUTHORIZED,
                KVError::NotFound  => StatusCode::NOT_FOUND,
            };
            return HttpResponse::from(e.to_string()).set_status(status);
        }
    };

    let value: Value = match deserialize_value(&record) {
        Ok(val) => val,
        Err(_) => {
            error!("Stored record is invalid");
            return HttpResponse::from("Stored record is invalid")
                .set_status(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // The stored URL
    let url = value.url;

    HttpResponse::from(String::from(&url))
        .set_status(StatusCode::MOVED_PERMANENTLY)
        .set_header("Location".parse().unwrap(), 
            HeaderValue::from_str(&url).unwrap())
}

