mod serverless_function;
mod value_parser;
use edjx::{error, HttpRequest, HttpResponse, StatusCode};

#[no_mangle]
pub fn init() {
    let req = match HttpRequest::from_client(true) {
        Ok(val) => val,
        Err(e) => {
            error!("{}", e.to_string().as_str());
            HttpResponse::new()
                .set_status(StatusCode::BAD_REQUEST)
                .send()
                .unwrap();
            return;
        }
    };

    let res = crate::serverless_function::serverless(req);
    match res.send() {
        Ok(x) => x,
        Err(e) => {
            error!("{}", e.to_string().as_str());
        }
    };
}
