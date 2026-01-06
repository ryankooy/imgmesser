import { apiPath } from "../store.ts";
import type { ImageMeta } from "../store.ts";

export function userLoginUrl(): string {
    return `${apiPath}/login`;
}

export function userRegisterUrl(): string {
    return `${apiPath}/register`;
}

export function imageUploadUrl(): string {
    return `${apiPath}/images`;
}

export function imageGalleryUrl(currentPage: number, limit: number): string {
    return `${apiPath}/images?page=${currentPage}&limit=${limit}`;
}

export function imageUrl(imageId: string): string {
    return `${apiPath}/images/${encodeURIComponent(imageId)}`;
}

export const getCurrentUser = async (): string | null => {
    try {
        const response = await fetch(`${apiPath}/user`);
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
        const response = await fetch(`${apiPath}/logout`, {
            method: "POST",
        });

        return response.ok;
    } catch (error) {
        console.error("Error fetching:", error);
    }

    return false;
};

export const getImageDataUrl = async (imageId: string): object | null => {
    try {
      const response = await fetch(imageUrl(imageId));
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

export const getImageMetadata = async (imageId: string): ImageMeta | null => {
    try {
      const response = await fetch(`${imageUrl(imageId)}/meta`);
      if (response.ok) {
        const data = await response.json();
        return data;
      }
    } catch (err) {
      console.error(`Failed to fetch image metadata:`, err);
    }

    return null;
}
