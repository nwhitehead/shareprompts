import { resolve } from 'path';
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

// https://vitejs.dev/config/
export default defineConfig({
    base: "",
    plugins: [vue()],
    build: {
        rollupOptions: {
            input: {
                popup: resolve(__dirname, 'popup.html'),
                index: resolve(__dirname, 'index.js'),
                background: resolve(__dirname, 'background.js'),
            },
            output: {
                entryFileNames: '[name].js',
            }
        }
    }
})
