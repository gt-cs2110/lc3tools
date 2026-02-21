import path from "node:path";
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import Components from 'unplugin-vue-components/vite';
import { PrimeVueResolver } from '@primevue/auto-import-resolver';
import Icons from 'unplugin-icons/vite';
import IconsResolver from 'unplugin-icons/resolver';

// https://vitejs.dev/config
export default defineConfig({
    server: {
        watch: {
            ignored: ['**/components.d.ts']
        }
    },
    plugins: [
        tailwindcss(),
        vue(),
        Components({
            resolvers: [
                PrimeVueResolver(),
                IconsResolver({
                    prefix: false,
                    enabledCollections: ['mdi']
                })
            ]
        }),
        Icons({
            scale: 1.5,
            compiler: 'vue3'
        })
    ],
    resolve: {
        alias: [
            { find: "@", replacement: path.resolve(__dirname, "./src") }
        ]
    }
});
