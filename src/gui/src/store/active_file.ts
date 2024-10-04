import { defineStore } from "pinia";

export const useActiveFileStore = defineStore("active_file", {
  state: () => ({
    path: null as string | null,
    lastLoaded: new Date(),
    lastBuilt: new Date()
  }),
  actions: {
    touchLoadTime() {
        this.lastLoaded = new Date();
    },
    touchBuildTime() {
        this.lastBuilt = new Date();
    },
  }
});