import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import("@sveltejs/vite-plugin-svelte").SvelteConfig} */
export default {
    // Consult https://svelte.dev/docs#compile-time-svelte-preprocess
    // for more information about preprocessors
    preprocess: [vitePreprocess({ script: true })],
    compilerOptions: { runes: true },
    onwarn: (warning, handler) => {
        // Suppress specific warnings
        if (warning.code.includes("a11y")) {
            return;
        }
        // Handle other warnings normally
        handler(warning);
    },
};
