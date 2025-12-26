import { writable } from "svelte/store";

export const currentView: string | null = writable(null);
export const currentUser: string | null = writable(null);
export const apiUrl: string = import.meta.env.VITE_API_URL;

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
    id: string;
    url: string;
    meta: ImageMeta;
}
