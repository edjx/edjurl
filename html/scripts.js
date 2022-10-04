function sendRequest() {
    // Set APP_URL to the URL of the deployed "shorten" function.
    // This function is called when the "Shorten" button
    // is clicked on the web page.
    //
    // Example:
    // const APP_URL = "https://5611543a-5ed7-458f-8014-11e19e0ddd12.fn.edjx.net/shorten";
    const APP_URL = "https://APPID.fn.edjx.net/FUNCTION";

    // Set FETCH_PREFIX to the URL of the deployed "fetch" function,
    // followed by the query parameter. The FETCH_PREFIX string is displayed
    // as a part of the short URL.
    //
    // Examples (before and after a domain is configured):
    // const FETCH_PREFIX = "https://5611543a-5ed7-458f-8014-11e19e0ddd12.fn.edjx.net/fetch?s=";
    // const FETCH_PREFIX = "https://edjurl.com/?s=";
    const FETCH_PREFIX = "https://DOMAIN/?s=";

    var url = document.getElementById("url").value;
    var alias = document.getElementById("alias").value;
    var password = document.getElementById("password").value;
    var old_password = document.getElementById("old_password").value;

    var params = {};
    if (url) {
        params.url = url;
    }
    if (alias) {
        params.alias = alias;
    }

    var headers = {};
    if (password) {
        headers.password = password;
    }
    if (old_password) {
        headers.old_password = old_password;
    }

    var status;
    fetch(APP_URL + "?" + new URLSearchParams(params), {
        method: "GET",
        headers: headers,
    })
    .then((response) => {
        status = response.status;
        return response.text();
    })
    .then((data) => {
        document.getElementById("container").style.display = "none";
        document.getElementById("container-result").style.display = "block";
        if (status == 200) {
            document.getElementById("result").value = FETCH_PREFIX + data;
            document.getElementById("div-result-ok").style.display = "block";
            document.getElementById("div-result-error").style.display = "none";
            document.getElementById("button-copy-url").style.display = "inline-block";
        } else {
            document.getElementById("error-message").innerText = "Server error " + status + ": " + data;
            document.getElementById("div-result-ok").style.display = "none";
            document.getElementById("div-result-error").style.display = "block";
            document.getElementById("button-copy-url").style.display = "none";
        }
    });

    // Do not submit the form
    return false;
}

function goBack() {
    document.getElementById("container").style.display = "block";
    document.getElementById("container-result").style.display = "none";
}

function copyURL() {
    var copyText = document.getElementById("result");

    copyText.select();
    copyText.setSelectionRange(0, 99999);

    navigator.clipboard.writeText(copyText.value)
    .catch(() => {
        alert("Copy to clipboard failed");
    });
}

function toggleAdvanced() {
    var div = document.getElementById("advanced");
    var button = document.getElementById("button-advanced");
    if (div.style.display === "none") {
        div.style.display = "block";
        button.classList.add("active");
    } else {
        div.style.display = "none";
        button.classList.remove("active");
    }
}

function togglePasswordChange() {
    var x = document.getElementById("old_password");
    var button = document.getElementById("button-password");
    if (x.style.display === "none") {
        x.style.display = "inline-block";
        button.classList.add("active");
    } else {
        x.style.display = "none";
        button.classList.remove("active");
    }
}