import { writable } from "svelte/store";

export const currentView = writable("login");
export const API_URL = "http://127.0.0.1:3000";
