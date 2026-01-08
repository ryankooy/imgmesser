const urlParams = new URLSearchParams(self.location.search);
const apiPath = urlParams.get("api_path");
const protectedUrls = ["/images", "/logout", "/user"];

// Prevent the worker from waiting until next
// page load to take over
self.addEventListener("activate", (event) => {
    event.waitUntil(clients.claim());
});

// IndexedDB storage for tokens
const storage = (() => {
    let dbInstance;

    function getDB() {
        if (dbInstance) return dbInstance;

        dbInstance = new Promise((resolve, reject) => {
            const openreq = indexedDB.open("tokenCache", 2);

            openreq.onerror = () => {
                reject(openreq.error);
            };
            openreq.onupgradeneeded = () => {
                openreq.result.createObjectStore("token");
            };
            openreq.onsuccess = () => {
                resolve(openreq.result);
            };
        });

        return dbInstance;
    }

    async function withStore(type, callback) {
        const db = await getDB();
        return new Promise((resolve, reject) => {
            const transaction = db.transaction("token", type);
            transaction.oncomplete = () => resolve();
            transaction.onerror = () => reject(transaction.error);
            callback(transaction.objectStore("token"));
        });
    }

    return {
        async get(key) {
            let request;
            await withStore("readonly", (store) => {
                request = store.get(key);
            });
            return request.result;
        },
        set(key, value) {
            return withStore("readwrite", (store) => {
                store.put(value, key);
            });
        },
        delete(key) {
            return withStore("readwrite", (store) => {
                store.delete(key);
            });
        },
    };
})();

function getRoutePath(path) {
    const parentPath = "/data";
    if (path.startsWith(parentPath)) {
        return path.substring(parentPath.length);
    }
    return path;
}

// Store tokens
async function setTokens(data) {
    await storage.set("tokens", {
        accessToken: data.access_token,
        refreshToken: data.refresh_token,
    });
}

// Request tokens from server
async function refreshTokens(tokens) {
    try {
        const response = await fetch(`${apiPath}/refresh`, {
            method: "POST",
            body: JSON.stringify({
                refresh_token: tokens.refreshToken,
            }),
            headers: { "Content-Type": "application/json" },
        });

        if (response.ok) {
            const data = await response.json();
            await setTokens(data);
        }
    } catch (error) {
        console.error("Failed to fetch refresh tokens:", error);
    }
}

// Build new request object from original request.
async function buildNewRequest(request, headers, body = null) {
    let newBody;

    if (body != null) {
        newBody = body;
    } else {
        try {
            // Try to parse the old request's body as JSON
            const responseJson = await request.json();
            newBody = JSON.stringify(responseJson);
        } catch (error) {
            // That didn't work, so use the ReadableStream
            newBody = request.body;
        }
    }

    return new Request(request.url, {
        method: request.method,
        headers: headers,
        credentials: request.credentials,
        cache: request.cache,
        redirect: request.redirect,
        referrer: request.referrer,
        body: newBody,
        context: request.context,
    });
}

// Update request with an Authorization header
async function updateRequest(request, route, tokens) {
    const r = request.clone();
    const headers = new Headers(Array.from(r.headers.entries()));

    // Add Authorization header with access token
    headers.append("Authorization", `Bearer ${tokens.accessToken}`);

    try {
        let body = null;

        if (route === "/logout") {
            body = JSON.stringify({
                refresh_token: tokens.refreshToken,
            });
            headers.append("Content-Type", "application/json");

            // User is logging out, so delete tokens
            await storage.delete("tokens");
        }

        // Build new request
        const newRequest = await buildNewRequest(r, headers, body);
        return newRequest;
    } catch (e) {
        console.error("Error making authorization request:", e);
    }

    return request;
}

async function interceptRequest(request) {
    const url = new URL(request.url);
    const route = getRoutePath(url.pathname);
    const isUploadRequest = route === "/images" && request.method === "POST";

    let tokens = await storage.get("tokens");

    const isProtectedUrl =
        !!tokens &&
        protectedUrls.some((path) => route.startsWith(path)) &&
        // We handle user authentication differently for image upload
        // requests, so we won't consider `/images` a protected URL
        // if the request message is POST
        !isUploadRequest;

    if (isProtectedUrl) {
        let newRequest;

        try {
            // Update request with an Authorization header
            newRequest = await updateRequest(request, route, tokens);
            const response = await fetch(newRequest);

            if (response.status === 401 && route !== "/user") {
                await refreshTokens(tokens);
                tokens = await storage.get("tokens");

                if (!!tokens) {
                    newRequest = await updateRequest(request, route, tokens);
                    return fetch(newRequest);
                }
            }

            return response;
        } catch (error) {
            console.error("Error fetching:", error);

            try {
                newRequest = await updateRequest(request, route, tokens);
                return fetch(newRequest);
            } catch (error) {
                console.error("Error fetching:", error);
            }
        }
    } else if (route === "/login") {
        try {
            const response = await fetch(request);
            const data = await response.json();

            // Stash the tokens from the response
            await setTokens(data);

            let newBody = {
                success: data.success,
                message: data.message,
            };

            if (data.user != null) newBody.user = data.user;

            return new Response(JSON.stringify(newBody), {
                status: response.status,
                statusText: response.statusText,
                headers: new Headers(Array.from(response.headers.entries())),
            });
        } catch (error) {
            console.error("Error fetching:", error);
        }
    }

    // Just return the original request if we got this far,
    // since that means one of the following is true:
    // * It's an image upload request
    // * We're missing tokens
    // * The URL is neither protected nor for authorization
    return fetch(request);
}

// Intercept all fetches
self.addEventListener("fetch", (event) => {
    event.respondWith(interceptRequest(event.request));
});

// If the app signals a page refresh, request tokens from the server
self.addEventListener("message", async (event) => {
    if (event.data && event.data.refresh) {
        const tokens = await storage.get("tokens");
        if (!!tokens) await refreshTokens(tokens);
    }
});
