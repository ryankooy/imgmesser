import { writable } from "svelte/store";

export const currentView = writable("login");
export const currentUser = writable(null);

export const apiUrl = "http://127.0.0.1:3000";

export interface ImageData {
    key: string;
    size: number;
    last_modified: string;
    content_type: string;
}

export const registerServiceWorker = async () => {
    if ("serviceWorker" in navigator) {
        try {
            const registration = await navigator.serviceWorker.register("worker.js");

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

export const getCurrentUser = async (): string | null => {
    try {
        const response = await fetch(`${apiUrl}/user`);
        const data = await response.json();

        if (response.ok) {
            return data.user.username;
        }
    } catch (error) {
        // Fail silently
    }

    return null;
};

export const logOut = async (): boolean => {
    try {
        const response = await fetch(`${apiUrl}/logout`, {
            method: "POST",
        });

        if (response.ok) return true;
    } catch (error) {
        console.error("Error fetching:", error);
    }

    return false;
};
