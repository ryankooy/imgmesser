const API_URL = "http://127.0.0.1:3000";
const authUrls = ["/login"];
const protectedUrls = ["/images"];

// Prevent the worker from waiting until next
// page load to take over
self.addEventListener("activate", (event) => {
    event.waitUntil(clients.claim());
});

let authToken = null;

function interceptRequest(request) {
    const url = new URL(request.url);
    const isApiOrigin = API_URL === url.origin;
    const isProtectedUrl = isApiOrigin && protectedUrls.includes(url.pathname);
    const isAuthUrl = isApiOrigin && authUrls.includes(url.pathname);

    if (authToken && isProtectedUrl) {
        console.log("authToken && isProtectedUrl");
        const headers = new Headers(Array.from(request.headers.entries()));

        // Attach token to header
        headers.append("Authorization", authToken);

        // Make a new request
        try {
            request = new Request(request.url, {
                method: request.method,
                headers: headers,
                credentials: request.credentials,
                cache: request.cache,
                redirect: request.redirect,
                referrer: request.referrer,
                body: request.body,
                context: request.context,
                //duplex: "half",
            });
        } catch (e) {
            console.error(e);
            // This will fail for CORS requests;
            // just continue with fetch caching
        }

        return fetch(request);
    } else if (isAuthUrl) {
        console.log("isAuthUrl");
        // Stash the token
        return fetch(request).then((response) =>
            response.json().then((data) => {
                authToken = data.access_token;

                const newBody = JSON.stringify({
                    success: data.success,
                    message: data.message,
                });

                return new Response(newBody, {
                    status: response.status,
                    statusText: response.statusText,
                    headers: new Headers(Array.from(response.headers.entries())),
                });
            }),
        );
    }

    console.log("default fetch");
    return fetch(request);
}

// Intercept all fetches
self.addEventListener("fetch", (event) => {
    event.respondWith(interceptRequest(event.request));
});
