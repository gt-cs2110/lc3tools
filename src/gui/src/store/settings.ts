import { defineStore } from "pinia";

export type LC3Settings = {
    theme: "light" | "dark",
    // Editor settings
    editor_binding: "standard" | "vim",
    autocomplete: "none" | "basic" | "full",
    soft_tabs: boolean,
    soft_tab_size: number,
    // Simulator settings
    numbers: "signed" | "unsigned",
    ignore_privilege: boolean,
    pause_on_fatal_trap: boolean,
    clear_out_on_reload: boolean,
    reduce_flashing: boolean,
}

export const useSettingsStore = defineStore("settings", {
    state: () => ({
        theme: "light",
        editor_binding: "standard",
        autocomplete: "full",
        soft_tabs: true,
        soft_tab_size: 4,
        numbers: "signed",
        ignore_privilege: false,
        pause_on_fatal_trap: true,
        clear_out_on_reload: true,
        reduce_flashing: false
    } as LC3Settings)
});