import { defineStore } from "pinia";

export type LC3Settings = {
    theme: "light" | "dark",
    numbers: "signed" | "unsigned",
    editor_binding: "standard" | "vim",
    ignore_privilege: boolean,
    liberal_asm: boolean,
    ignore_update: boolean,
    run_until_halt: boolean,
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
        run_until_halt: true,
        clear_out_on_reload: true,
        autocomplete: "full"
    } as LC3Settings)
});