import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import("@sveltejs/vite-plugin-svelte").SvelteConfig} */
export default {
  // Consult https://svelte.dev/docs#compile-time-svelte-preprocess
  // for more information about preprocessors
    preprocess: [vitePreprocess({ script: true })],
    onwarn: (warning, handler) => {
        // Suppress specific warnings
        if (warning.code === "a11y-click-events-have-key-events") return;
        if (warning.code === "a11y-no-static-element-interactions") return;

        // Handle other warnings normally
        handler(warning);
    },
};
