// Register the service worker.
export const registerServiceWorker = async () => {
    if ("serviceWorker" in navigator) {
        try {
            const registration = await navigator.serviceWorker.register(
                "worker.js"
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

// Inform the service worker of a page refresh.
export const handlePageRefresh = async () => {
    if (performance.navigation.type === 1 && navigator.serviceWorker) {
        // The page was refreshed; send the service worker a
        // REFRESH message so that if the user's logged in,
        // we can keep them logged in
        navigator.serviceWorker.ready.then(async (registration) => {
            if (registration.active) {
                registration.active.postMessage({ refresh: true });
            }
        });
    }
}

// Get the extension of a filename; the period/dot is not included.
export function getFileExtension(filename: string): string {
    return (filename.indexOf(".") !== -1) ? filename.split(".").pop() : "jpg";
}

// Strip the period/dot and extension off of a filename.
export function getFileStem(filename: string): string {
    if (filename.indexOf(".") === -1) return filename;
    return filename.substring(0, filename.lastIndexOf("."));
}

// Truncate the stem of a filename, appending an ellipsis
// and the file extension.
export function truncateFileName(val: string): string {
    if (val.length < 26) return val;
    return `${val.substring(0, 22)}... .${getFileExtension(val)}`;
}

// Get the display value for an image's content type.
export function formatImageType(contentType: string): string {
    if (!contentType.startsWith("image/")) return "UNKNOWN";
    return contentType.split("/").pop().toUpperCase();
}

export function formatFileSize(bytes: number): string {
    if (bytes < 1024) return bytes + " B";
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
    return (bytes / (1024 * 1024)).toFixed(1) + " MB";
}

export function formatDate(dateStr: string): string {
    try {
        return new Date(dateStr).toLocaleString();
    } catch {
        return dateStr;
    }
}
