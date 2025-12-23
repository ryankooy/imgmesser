import { apiUrl } from "../store.ts";

export const registerServiceWorker = async () => {
    if ("serviceWorker" in navigator) {
        try {
            const registration = await navigator.serviceWorker.register(
                "worker.js?api_url=" + apiUrl
            );

            if (registration.installing) {
                console.log("Service worker installing");
            } else if (registration.waiting) {
                console.log("Service worker installed and waiting");
            } else if (registration.active) {
                console.log("Service worker active");
            }

        } catch (error) {
            console.error("Service worker registration failed:", error);
        }
    }
};

export const handlePageRefresh = async () => {
    if (performance.navigation.type === 1 && navigator.serviceWorker) {
        // The page was refreshed; send the service worker a
        // REFRESH message so that if the user's logged in,
        // we can keep them logged in
        navigator.serviceWorker.ready.then(async (registration) => {
            if (registration.active) {
                registration.active.postMessage({ type: "REFRESH" });
            }
        });
    }
}
