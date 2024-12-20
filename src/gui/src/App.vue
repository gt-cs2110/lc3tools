<template>
  <div id="app">
    <v-app
      id="lc3tools"
      :theme="settings.theme"
    >
      <v-app-bar
        density="compact"
        :elevation="2"
      >
        <v-app-bar-title>
          <strong>LC3</strong>Tools
          <!-- Put buttons next to title -->
          <v-btn
            icon
            flat
          >
            <v-icon :icon="mdiCog" />
            <v-menu
              activator="parent"
              :close-on-content-click="false"
            >
              <v-card>
                <v-container class="d-flex flex-column ga-3">
                  <!-- Should use v-row, v-col, but those are grid not flex -->
                  <div class="d-flex justify-space-between align-center">
                    <h3 class="flex-grow-1">
                      Theme
                    </h3>
                    <v-radio-group
                      v-model="settings.theme"
                      class="flex-shrink-1"
                      color="primary"
                      inline
                      hide-details
                      @change="saveSettings('theme')"
                    >
                      <v-spacer />
                      <v-radio
                        label="Light"
                        value="light"
                      />
                      <v-radio
                        label="Dark"
                        value="dark"
                      />
                    </v-radio-group>
                  </div>
                  <v-divider />
                  <div class="d-flex justify-space-between align-center">
                    <h3 class="flex-grow-1">
                      Editor Key Binding
                    </h3>
                    <v-radio-group
                      v-model="settings.editor_binding"
                      class="flex-shrink-1"
                      color="primary"
                      inline
                      hide-details
                      @change="saveSettings('editor_binding')"
                    >
                      <v-spacer />
                      <v-radio
                        label="Standard"
                        value="standard"
                      />
                      <v-radio
                        label="Vim"
                        value="vim"
                      />
                    </v-radio-group>
                  </div>
                  <div class="d-flex justify-space-between align-center">
                    <h3 class="flex-grow-1">
                      Autocomplete
                    </h3>
                    <v-radio-group
                      v-model="settings.autocomplete"
                      class="flex-shrink-1"
                      color="primary"
                      inline
                      hide-details
                      @change="saveSettings('autocomplete')"
                    >
                      <v-spacer />
                      <v-radio
                        label="None"
                        value="none"
                      />
                      <v-radio
                        label="Basic"
                        value="basic"
                      />
                      <v-radio
                        label="Full"
                        value="full"
                      />
                    </v-radio-group>
                  </div>
                  <div class="d-flex justify-space-between align-center">
                    <h3 class="flex-grow-1">
                      Soft Tabs
                    </h3>
                    <v-checkbox 
                      v-model="settings.soft_tabs"
                      hide-details
                    />
                    <v-text-field 
                      v-model.number="settings.soft_tab_size"
                      variant="outlined" 
                      hide-details 
                      density="compact"
                      type="number"
                      width="3"
                      :disabled="!settings.soft_tabs"
                    />
                  </div>
                  <v-divider />
                  <div class="d-flex justify-space-between align-center">
                    <h3 class="flex-grow-1">
                      Number View
                    </h3>
                    <v-radio-group
                      v-model="settings.numbers"
                      class="flex-shrink-1"
                      color="primary"
                      inline
                      hide-details
                      @change="saveSettings('numbers')"
                    >
                      <v-spacer />
                      <v-radio
                        label="Unsigned"
                        value="unsigned"
                      />
                      <v-radio
                        label="Signed"
                        value="signed"
                      />
                    </v-radio-group>
                  </div>
                  <div class="d-flex justify-space-between align-center">
                    <h3 class="flex-grow-1">
                      Pause on HALT and exceptions
                    </h3>
                    <v-switch
                      v-model="settings.pause_on_fatal_trap"
                      class="flex-shrink-1"
                      color="primary"
                      hide-details
                      @change="saveSettings('pause_on_fatal_trap')"
                    />
                  </div>
                  <div class="d-flex justify-space-between align-center">
                    <h3 class="flex-grow-1">
                      Clear output on object file reload
                    </h3>
                    <v-switch
                      v-model="settings.clear_out_on_reload"
                      class="flex-shrink-1"
                      color="primary"
                      hide-details
                      @change="saveSettings('clear_out_on_reload')"
                    />
                  </div>
                  <div class="d-flex justify-space-between align-center">
                    <div class="flex-grow-1">
                      <h3>Ignore privileged mode</h3>
                      <p
                        v-if="settings.ignore_privilege"
                        class="text-red"
                      >
                        May result in inconsistency with the grader.
                      </p>
                    </div>
                    <v-switch
                      v-model="settings.ignore_privilege"
                      class="flex-shrink-1"
                      color="primary"
                      hide-details
                      @change="saveSettings('ignore_privilege')"
                    />
                  </div>
                  <div class="d-flex justify-space-between align-center">
                    <h3 class="flex-grow-1">
                      Reduce flashing in simulator
                    </h3>
                    <v-switch
                      v-model="settings.reduce_flashing"
                      class="flex-shrink-1"
                      color="primary"
                      hide-details
                      @change="saveSettings('reduce_flashing')"
                    />
                  </div>
                  <v-divider />
                  <div class="d-flex justify-center">
                    <h4>Issues? Post on CS 2110 Ed/Piazza!</h4>
                  </div>
                </v-container>
              </v-card>
            </v-menu>
          </v-btn>
        </v-app-bar-title>
        <v-tabs>
          <v-tab
            exact
            to="/editor"
            icon
          >
            <v-icon
              size="x-large"
              :icon="mdiCodeTags"
            />
            <v-tooltip
              location="bottom"
              activator="parent"
              text="Editor"
            />
          </v-tab>
          <v-tab
            exact
            to="/simulator"
            icon
          >
            <v-icon
              size="x-large"
              :icon="mdiMemory"
            />
            <v-tooltip
              location="bottom"
              activator="parent"
              text="Simulator"
            />
          </v-tab>
        </v-tabs>
      </v-app-bar>

      <router-view v-slot="{ Component }">
        <keep-alive>
          <component :is="Component" />
        </keep-alive>
      </router-view>
    </v-app>
  </div>
</template>
  
<script setup lang="ts">
// Vue stuff
import { onMounted } from "vue";
import "vuetify/components";
import { mdiCog, mdiCodeTags, mdiMemory } from "@mdi/js";
import { LC3Settings, useSettingsStore } from "./store/settings";

const { lc3, storage } = window.api;

// Settings
const settings = useSettingsStore();
settings.$patch(storage.getAll());

const lc3SettingCalls = {
  "ignore_privilege": lc3.setIgnorePrivilege,
  "pause_on_fatal_trap": lc3.setPauseOnFatalTrap
} satisfies Partial<Record<keyof LC3Settings, (status: boolean) => void>>;

onMounted(() => {
  for (const [key, f] of Object.entries(lc3SettingCalls)) {
    f(settings[key as keyof typeof lc3SettingCalls]);
  }
})

// Settings
type SettingKeys = keyof LC3Settings | "all";
function saveSettings(setting: SettingKeys) {
  if (setting === "all") {
    for (const [key, f] of Object.entries(lc3SettingCalls)) {
      f(settings[key as keyof typeof lc3SettingCalls]);
    }
    storage.setAll(settings);
  } else {
    if (setting in lc3SettingCalls) {
      const s = setting as keyof typeof lc3SettingCalls;
      lc3SettingCalls[s](settings[s]);
    }
    storage.set(setting, settings[setting]);
  }
}
</script>

<style lang="css">
/* dark mode autocomplete menu tweaks ------------------------------ */
.ace_editor.ace_autocomplete.ace_dark {
    background-color: #141414 !important;
    color: white !important;
    border-color: gray !important;
    color-scheme: dark;
}

.ace_editor.ace_autocomplete.ace_dark .ace_marker-layer .ace_active-line {
    background-color: hsl(223, 30%, 19%) !important;
}

.ace_editor.ace_autocomplete.ace_dark .ace_line .ace_completion-highlight {
    color: hsl(214deg 100% 66%);
}

/*  */
.application {
    font-size: 1em;
}

/* Other stuff */
body {
    user-select: none;
    cursor: default;
}

/* https://stackoverflow.com/q/56973002/11984788 */
html {
    overflow-y: auto !important;
}

.d-contents {
  display: contents;
}
</style>