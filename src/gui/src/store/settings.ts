import { defineStore } from "pinia";

export type LC3Settings = {
    theme: "light" | "dark",
    numbers: "signed" | "unsigned",
    editor_binding: "standard" | "vim",
    ignore_privilege: boolean,
    ignore_update: boolean,
    pause_on_fatal_trap: boolean,
    clear_out_on_reload: boolean,
    autocomplete: "none" | "basic" | "full"
}

export const useSettingsStore = defineStore("settings", {
    state: () => ({
        theme: "light",
        numbers: "signed",
        editor_binding: "standard",
        ignore_privilege: false,
        ignore_update: false,
        pause_on_fatal_trap: true,
        clear_out_on_reload: true,
        autocomplete: "full"
    } as LC3Settings)
});