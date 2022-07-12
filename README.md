# EDJX Platform Example App (EdjURL)

This is an example application that demonstrates the features of the EDJX platform.

## App Description

This app implements a simple URL shortener.

![EdjURL Screenshot](/screenshot.png)

The app consists of these components:

- URL shortening function
    - Source code: [rust_url_shorten](/rust_url_shorten)
    - Takes a URL and generates a short string
    - The URL is passed to the function in a `url` query parameter
- URL fetch function
    -  Source code: [rust_url_fetch](/rust_url_fetch) or [cpp_url_fetch](/cpp_url_fetch)
    - Given a short string, it returns the full URL
    The short string is passed to the function in a `s` query parameter
- HTML page
    - Source code: [html](/html)
    - Provides a user interface for the functions

## How to Deploy the App

1. Deploy the application code
    1. Create a new `edjurl` application in EdjConsole
    2. Build the [rust_url_shorten](/rust_url_shorten) code to get a `urlshorten.wasm` file
    3. Create a new `shorten` function inside the `edjurl` application in EdjConsole, select the `urlshorten.wasm` file
    4. Build the [rust_url_fetch](/rust_url_fetch) (or [cpp_url_fetch](/cpp_url_fetch)) code to get a `urlfetch.wasm` file
    5. Create a new `fetch` function inside the `edjurl` application in EdjConsole, select the `urlfetch.wasm` file
2. Deploy the HTML page
    1. In the [html/scripts.js](/html/scripts.js) file, change
        - the `APP_URL` variable to the URL of the deployed `shorten` function
        - the `FETCH_PREFIX` variable to the URL of the deployed `fetch` function followed by the `?s=` query string
    1. Create a new `edjurl` bucket in EdjConsole
    2. Upload all files from the [html](/html) folder to the new `edjurl` bucket
    3. In EdjConsole, set the `Content-Disposition` header to the value `inline` for the `index.html` file
3. Add a domain
    1. If you own a domain (e.g., `edjurl.com`), you can add it to EdjConsole
    2. You can set Request Routing rules in EdjConsole, so that:
        - `http://edjurl.com` (exact match) redirects to the HTML page in the bucket `https://[BucketID].storage.edjx.net/blobs/index.html`
        - `http://edjurl.com/?s=*` (pattern match) redirects to `https://[AppID].fn.edjx.net/fetch?{query}`