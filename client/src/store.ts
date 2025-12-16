import { writable } from "svelte/store";

export const currentView: string = writable("login");
export const currentUser: string | null = writable(null);

export const apiUrl: string = import.meta.env.VITE_API_URL;

export interface ImageData {
    id: string;
    name: string;
    content_type: string;
    created_at: string;
    last_modified: string;
    version: string;
    width: number;
    height: number;
    size: number;
    version_count: number;
    version_index: number;
    latest_version: boolean;
    initial_version: boolean;
}

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
            headers: {"Content-Type": "application/json"},
        });

        return response.ok;
    } catch (error) {
        console.error("Error fetching:", error);
    }

    return false;
};

export function truncateFileName(val: string): string {
    const ext = (val.indexOf(".") !== -1) ? val.split(".").pop() : "";
    return (val.length > 25) ? val.substring(0, 22) + `... .${ext}` : val;
}
