import { writable } from "svelte/store";

export const currentView: string | null = writable(null);
export const currentUser: string | null = writable(null);

export const apiUrl: string = "/data";

// Metadata from the server
export interface ImageMeta {
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

export interface ImageData {
    // Image ID
    id: string;

    // Image data URL
    url: string;

    // Image metadata
    meta: ImageMeta;
}
