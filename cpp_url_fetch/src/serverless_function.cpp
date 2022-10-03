#include <cstdint>
#include <string>
#include <vector>
#include <optional>

#include <edjx/logger.hpp>
#include <edjx/request.hpp>
#include <edjx/response.hpp>
#include <edjx/error.hpp>
#include <edjx/kv.hpp>
#include <edjx/http.hpp>

#include "value_parser.hpp"

using edjx::logger::info;
using edjx::logger::error;
using edjx::request::HttpRequest;
using edjx::response::HttpResponse;
using edjx::error::KVError;
using edjx::http::HttpStatusCode;

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
//   - HTTP response with an HTTP 301 redirect to the stored URL
//   On failure:
//   - HTTP response with a 4xx or 5xx HTTP status code and an error message
//     in the body.
//

static const HttpStatusCode HTTP_STATUS_NO_CONTENT = 204;
static const HttpStatusCode HTTP_STATUS_MOVED_PERMANENTLY = 301;
static const HttpStatusCode HTTP_STATUS_BAD_REQUEST = 400;
static const HttpStatusCode HTTP_STATUS_UNAUTHORIZED = 401;
static const HttpStatusCode HTTP_STATUS_NOT_FOUND = 404;
static const HttpStatusCode HTTP_STATUS_METHOD_NOT_ALLOWED = 405;
static const HttpStatusCode HTTP_STATUS_INTERNAL_SERVER_ERROR = 500;

std::optional<std::string> query_param_by_name(const HttpRequest & req, const std::string & param_name) {
    std::string uri = req.get_uri().as_string();
    std::vector<std::pair<std::string, std::string>> query_parsed;

    // e.g., https://example.com/path/to/page?name=ferret&color=purple

    size_t query_start = uri.find('?');

    if (query_start != std::string::npos) {
        // Query is present
        std::string name;
        std::string value;
        bool parsing_name = true;
        for (std::string::iterator it = uri.begin() + query_start + 1; it != uri.end(); ++it) {
            char c = *it;
            switch (c) {
                case '?':
                    break; // Invalid URI
                case '=':
                    parsing_name = false;
                    break;
                case '&':
                    query_parsed.push_back(make_pair(name, value));
                    name.clear();
                    value.clear();
                    parsing_name = true;
                    break;
                default:
                    if (parsing_name) {
                        name += c;
                    } else {
                        value += c;
                    }
                    break;
            }
        }
        if (!name.empty() || !value.empty()) {
            query_parsed.push_back(make_pair(name, value));
        }

        for (const auto & parameter : query_parsed) {
            if (parameter.first == param_name) {
                return parameter.second;
            }
        }
    }

    return {};
}

HttpResponse serverless(const HttpRequest & req) {
    info("URL decoding function started");

    // CORS preflight workaround for web browsers
    if (req.get_method() == edjx::http::HttpMethod::OPTIONS) {
        info("Web browser sent an OPTIONS request. Responding with CORS data...");
        return HttpResponse()
            //.set_header("Access-Control-Allow-Origin", "*")
            .set_header("Access-Control-Allow-Methods", "GET")
            .set_header("Access-Control-Allow-Headers", "*")
            .set_status(HTTP_STATUS_NO_CONTENT);
    }

    if (req.get_method() != edjx::http::HttpMethod::GET) {
        return HttpResponse("GET method expected")
            .set_status(HTTP_STATUS_METHOD_NOT_ALLOWED);
    }

    // Short string that represents a stored URL ("s" query parameter)
    std::optional<std::string> shortstr = query_param_by_name(req, "s");
    if (!shortstr.has_value()) {
        error("No key provided in user request");
        return HttpResponse("No key provided in user request")
            .set_status(HTTP_STATUS_BAD_REQUEST);
    }

    // Retrieve the original URL from the KV store
    std::vector<uint8_t> record;
    KVError err = edjx::kv::get(record, shortstr.value());
    switch (err) {
        case KVError::Unknown:
            return HttpResponse(edjx::error::to_string(err))
                .set_status(HTTP_STATUS_BAD_REQUEST);
        case KVError::UnAuthorized:
            return HttpResponse(edjx::error::to_string(err))
                .set_status(HTTP_STATUS_UNAUTHORIZED);
        case KVError::NotFound:
            return HttpResponse(edjx::error::to_string(err))
                .set_status(HTTP_STATUS_NOT_FOUND);
        default:
            break;
    }

    // Decode the KV entry
    Value value;
    if (!deserialize_value(value, record)) {
        error("Stored record is invalid");
        return HttpResponse("Stored record is invalid")
                .set_status(HTTP_STATUS_INTERNAL_SERVER_ERROR);
    }

    // Get the stored URL
    std::string url = value.url;

    return HttpResponse(url)
        .set_status(HTTP_STATUS_MOVED_PERMANENTLY)
        .set_header("Location", url);
}