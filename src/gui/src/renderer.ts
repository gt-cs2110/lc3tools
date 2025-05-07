/**
 * This file will automatically be loaded by vite and run in the "renderer" context.
 * To learn more about the differences between the "main" and the "renderer" context in
 * Electron, visit:
 *
 * https://electronjs.org/docs/tutorial/application-architecture#main-and-renderer-processes
 *
 * By default, Node.js integration in this file is disabled. When enabling Node.js integration
 * in a renderer process, please be aware of potential security implications. You can read
 * more about security risks here:
 *
 * https://electronjs.org/docs/tutorial/security
 *
 * To enable Node.js integration in this file, open up `main.ts` and enable the `nodeIntegration`
 * flag:
 *
 * ```
 *  // Create the browser window.
 *  mainWindow = new BrowserWindow({
 *    width: 800,
 *    height: 600,
 *    webPreferences: {
 *      nodeIntegration: true
 *    }
 *  });
 * ```
 */

import { createApp } from 'vue';
import App from './App.vue';
import router from "./router/index";

// Theming
import PrimeVue, { PrimeVueConfiguration } from 'primevue/config';
import ToastService from 'primevue/toastservice';
import Aura from '@primevue/themes/aura';
import "./style.css";

// Pinia
import { createPinia } from 'pinia';
import { definePreset } from '@primevue/themes';
const pinia = createPinia();

createApp(App)
    .use(router)
    .use(PrimeVue, {
        theme: {
            preset: definePreset(Aura, {
                semantic: {
                    colorScheme: {
                        light: {
                            surface: {
                                0: '#ffffff',
                                50: '{zinc.50}',
                                100: '{zinc.100}',
                                200: '{zinc.200}',
                                300: '{zinc.300}',
                                400: '{zinc.400}',
                                500: '{zinc.500}',
                                600: '{zinc.600}',
                                700: '{zinc.700}',
                                800: '{zinc.800}',
                                900: '{zinc.900}',
                                950: '{zinc.950}'
                            },
                            content: {
                                hoverBackground: '{surface.300}',
                                borderColor: '{surface.200}'
                            }
                        },
                        dark: {
                            surface: {
                                0: '#ffffff',
                                50: '{zinc.50}',
                                100: '{zinc.100}',
                                200: '{zinc.200}',
                                300: '{zinc.300}',
                                400: '{zinc.400}',
                                500: '{zinc.500}',
                                600: '{zinc.600}',
                                700: '{zinc.700}',
                                800: '{zinc.800}',
                                900: '{zinc.900}',
                                950: '{zinc.950}'
                            },
                            content: {
                                hoverBackground: '{surface.700}',
                                borderColor: '{surface.800}'
                            }
                        }
                    }
                }
            }),
            options: {
                darkModeSelector: ".dark",
                cssLayer: {
                    name: 'primevue',
                    order: 'theme, base, primevue'
                }
            },
        },
        pt: {
            global: {
                css: `
                    /* Hide badges if the "hide-badge" class is added to a badge or an overlay badge */
                    .p-overlaybadge.hide-badge .p-badge, .p-badge.hide-badge {
                        opacity: 0;
                    }
                `,
            },
            // Make badges opacity transition
            badge: {
                root: 'transition-opacity',
            }
        }
    } satisfies PrimeVueConfiguration)
    .use(ToastService)
    .use(pinia)
    .mount('#app');