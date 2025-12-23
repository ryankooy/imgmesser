import { apiUrl } from "../store.ts";

export const getCurrentUser = async (): string | null => {
    try {
        const response = await fetch(`${apiUrl}/user`);
        if (response.ok) {
            const data = await response.json();
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

export const getImageDataUrl = async (imageId: string): object | null => {
    try {
      const response = await fetch(`${apiUrl}/images/${encodeURIComponent(imageId)}`);
      if (response.ok) {
        const blob = await response.blob();
        const dataUrl = URL.createObjectURL(blob);
        return dataUrl;
      }
    } catch (err) {
      console.error(`Failed to fetch image data:`, err);
    }

    return null;
}

export const getImageMetadata = async (imageId: string): ImageData | null => {
    try {
      const response = await fetch(`${apiUrl}/images/${encodeURIComponent(imageId)}/meta`);
      if (response.ok) {
        const data = await response.json();
        return data;
      }
    } catch (err) {
      console.error(`Failed to fetch image metadata:`, err);
    }

    return null;
}
