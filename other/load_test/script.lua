--- t1 -10 -d1s -s script.lua http://localhost:8080
--

wrk.method = "GET"
wrk.path = "/node_modules/@sveltejs/kit/src/runtime/client/client.js?v=e32e175e"
-- wrk.path = "/"


wrk.headers = {
    ["Host"] = "0001.localhost:8080",
    ["Accept"] = "*/*",
    ["Accept-Language"] = "en-US,en;q=0.9",
    ["Connection"] = "keep-alive",
    ["Origin"] = "http://0001.localhost:8080",
    ["Referer"] =
    "http://0001.localhost:8080/",
    ["Sec-Fetch-Dest"] = "script",
    ["Sec-Fetch-Mode"] = "cors",
    ["Sec-Fetch-Site"] = "same-origin",
    ["User-Agent"] =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36",
    -- ["sec-ch-ua"] = [["Chromium";v="140", "Not=A?Brand";v="24", "Google Chrome";v="140"]],
    -- ["sec-ch-ua-mobile"] = "?0",
    ["sec-ch-ua-platform"] = [["macOS"]],
}
