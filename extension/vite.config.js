import { resolve } from 'path';
import { defineConfig } from 'vite';
import { crx } from '@crxjs/vite-plugin'
import manifest from './manifest.json' assert { type: 'json' } // Node >=17
import vue from '@vitejs/plugin-vue';

// https://vitejs.dev/config/
export default defineConfig({
    base: "",
    plugins: [
        vue(),
        crx({ manifest })
    ],
})
