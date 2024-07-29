import { defineStore } from "pinia";

export const useActiveFileStore = defineStore("active_file", {
  state: () => ({
    path: null as string | null,
    last_loaded: new Date(),
    last_built: new Date()
  }),
  actions: {
    touchLoadTime() {
        this.last_loaded = new Date();
    },
    touchBuildTime() {
        this.last_built = new Date();
    },
  }
});