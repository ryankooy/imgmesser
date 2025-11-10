import { writable } from "svelte/store";

export const currentView = writable("login");
export const userLoggedIn = writable(false);

export const API_URL = "http://127.0.0.1:3000";

export const registerServiceWorker = async () => {
    if ("serviceWorker" in navigator) {
        try {
            const registration = await navigator.serviceWorker.register(
                "../worker.js",
            );

            if (registration.installing) {
                console.log("Service worker installing");
            } else if (registration.waiting) {
                console.log("Service worker installed and waiting");
            } else if (registration.active) {
                console.log(`Service worker active: ${registration.active.scriptURL}`);
            }

        } catch (error) {
            console.error(`Registration failed with ${error}`);
        }
    }
};
