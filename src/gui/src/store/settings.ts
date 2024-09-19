import { defineStore } from "pinia";

export type LC3Settings = {
    theme: "light" | "dark",
    numbers: "signed" | "unsigned",
    editor_binding: "standard" | "vim",
    ignore_privilege: boolean,
    liberal_asm: boolean,
    ignore_update: boolean,
    pause_on_fatal_trap: boolean,
    strict_mem_accesses: boolean,
    clear_out_on_reload: boolean,
    autocomplete: "none" | "basic" | "full"
}

export const useSettingsStore = defineStore("settings", {
    state: () => ({
        theme: "light",
        numbers: "signed",
        editor_binding: "standard",
        ignore_privilege: false,
        liberal_asm: false,
        ignore_update: false,
        pause_on_fatal_trap: true,
        strict_mem_accesses: false,
        clear_out_on_reload: true,
        autocomplete: "full"
    } as LC3Settings)
});