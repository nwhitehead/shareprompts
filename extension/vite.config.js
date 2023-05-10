import { defineConfig } from 'vite';
import manifest from './manifest.json' assert { type: 'json' } // Node >=17
import vue from '@vitejs/plugin-vue';
import webExtension from "@samrum/vite-plugin-web-extension";

// https://vitejs.dev/config/
export default defineConfig({
    base: "",
    plugins: [
        vue(),
        webExtension({ manifest }),
    ],
})
