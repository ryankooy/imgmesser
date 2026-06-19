import { writable } from "svelte/store";

export const currentView: string | null = writable(null);
export const currentUser: string | null = writable(null);
export const galleryPageCache: Map<number, GalleryPageInfo> = writable(new Map());
export const imageDataUrlCache: Map<string, string> = writable(new Map());

export const apiPath: string = (import.meta.env.PROD) ? "/data" : "http://127.0.0.1:3000";

export interface GalleryPageInfo {
    images: ImageMeta[],
    total: number,
    has_more: boolean,
}

export interface GalleryPagination {
    current_page: number;
    has_more: boolean;
}

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
    original_version: string;
}

// Image id, data URL, and database metadata
export interface ImageData {
    // Image ID
    id: string;

    // Image data URL
    url: string;

    // Image metadata
    meta: ImageMeta;
}

export interface Resize {
    width: number;
    height: number;
}

export interface Crop {
    width: number;
    height: number;
    x: number;
    y: number;
}

export interface Options {
    morphology: string;
    mask: string;
    radius: number | null;
}

export interface Filters {
    grayscale: boolean;
    sepia: boolean;
    options: Options | null;
}

// Image transformation specifications
export interface Transformations {
    resize: Resize | null;
    crop: Crop | null;
    rotate: number | null;
format: string | null;
    filters: Filters | null;
}

// Icon menu item properties
export interface IconMenuItem {
    title: string;
    iconName: string;
    handleClick: (() => void) | null;
}

// Icon menu properties
export interface IconMenu {
    title: string;
    iconName: string;
    handleClick: (() => void) | null;
    handleClickOutside: (() => void) | null;
    toggleFunc: ((boolean) => void) | null;
    items: IconMenuItem[];
}

// Image statuses
export const enum ImageStatus {
    None,
    Loading,
    Transforming,
    Panning,
    Closing,
    Saving,
    Canceling,
    Deleting,
}

// Image editing statuses
export const enum EditStatus {
    None,
    Rotating,
    Cropping,
    Resizing,
    SettingFilters,
}
