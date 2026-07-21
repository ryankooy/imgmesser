import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { readFileSync } from "fs";
import { fileURLToPath } from "url";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

const file = fileURLToPath(new URL("package.json", import.meta.url));
const pkg = JSON.parse(readFileSync(file, "utf8"));

export default defineConfig({
    plugins: [
        wasm(),
        topLevelAwait(),
        svelte(),
    ],
    server: {
        host: "127.0.0.1",
        port: 5173,
    },
    define: {
        PKG: pkg,
    },
    optimizeDeps: {
        exclude: ["transformjs"]
    }
});
