import { defineConfig } from 'vite';
import manifest_chrome from './chrome/manifest.json' assert { type: 'json' } // Node >=17
import manifest_firefox from './firefox/manifest.json' assert { type: 'json' } // Node >=17
import vue from '@vitejs/plugin-vue';
import webExtension from "@samrum/vite-plugin-web-extension";

const is_firefox = (process.env.BROWSER === 'firefox');
const manifest = is_firefox ? manifest_firefox : manifest_chrome;
const outDir = is_firefox ? "dist/firefox/" : "dist/chrome/";

export default defineConfig({
    base: "",
    build: {
        minify: false,
        outDir
    },
    plugins: [
        vue(),
        webExtension({ 
            manifest,
            useDynamicUrlWebAccessibleResources: false,
        }),
    ],
})
