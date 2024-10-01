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
            v-if="update_available"
            icon
            flat
            @click="downloadUpdate()"
          >
            <v-icon
              color="green"
              :icon="mdiInformation"
            />
            <v-tooltip
              location="bottom"
              activator="parent"
              text="Update"
            />
          </v-btn>
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
                <v-container class="d-flex flex-column ga-5">
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
          <!-- Wrap in a div so that element can always be switched out -->
          <div class="d-contents">
            <component :is="Component" />
          </div>
        </keep-alive>
      </router-view>

      <v-dialog
        v-model="update_dialog"
        max-width="400"
        persistent
      >
        <v-card>
          <v-card-title
            v-if="!download_bar"
            class="headline"
          >
            Update Available
          </v-card-title>

          <v-card-text>
            {{
              download_bar
                ? "Downloading at " +
                  (update.download_speed / 1024).toFixed(0) +
                  " KB/s"
                : "Would you like to update now?"
            }}
            <v-progress-linear
              v-if="download_bar"
              :model-value="
                (update.download_transferred / update.download_size) * 100
              "
            />
          </v-card-text>

          <v-card-actions v-if="!download_bar">
            <v-btn
              icon
              flat
              @click="ignoreUpdate()"
            >
              <v-icon :icon="mdiDelete" />
              <v-tooltip
                location="top"
                activator="parent"
                text="Ignore"
              />
            </v-btn>

            <v-btn
              icon
              flat
              @click="update_dialog = false"
            >
              <v-icon
                :icon="mdiThumbDown"
                color="red-darken-1"
              />
              <v-tooltip
                location="top"
                activator="parent"
                text="No"
              />
            </v-btn>

            <v-btn
              icon
              flat
              @click="updateConfirmed()"
            >
              <v-icon
                :icon="mdiThumbUp"
                color="green-darken-1"
              />
              <v-tooltip
                location="top"
                activator="parent"
                text="Yes"
              />
            </v-btn>
          </v-card-actions>
        </v-card>
      </v-dialog>
    </v-app>
  </div>
</template>
  
<script setup lang="ts">
// Vue stuff
import { onMounted, ref } from "vue";
import "vuetify/components";
import { mdiInformation, mdiCog, mdiCodeTags, mdiMemory, mdiDelete, mdiThumbDown, mdiThumbUp } from "@mdi/js";
import { LC3Settings, useSettingsStore } from "./store/settings";

const { lc3, autoUpdater, storage } = window.api;

// Update download progress
const update = ref({
  download_speed: 0,
  download_transferred: 0,
  download_size: 0
});
const update_dialog = ref(false);
const update_available = ref(false);
const download_bar = ref(false);

// Settings
const settings = useSettingsStore();
settings.$patch(storage.getAll());

const lc3SettingCalls = {
  "ignore_privilege": lc3.setIgnorePrivilege,
  "pause_on_fatal_trap": lc3.setPauseOnFatalTrap
} satisfies Partial<Record<keyof LC3Settings, (status: boolean) => void>>;

onMounted(() => {
  autoUpdater.on((message, progress) => {
    if (message === "update_available") {
        // Show the settings modal
        update_dialog.value = !settings.ignore_update;
        update_available.value = true;
      }
      if (message === "download_progress") {
        update.value.download_speed = progress.bytesPerSecond;
        update.value.download_size = progress.total;
        update.value.download_transferred = progress.transferred;
      }
  })

  for (let [key, f] of Object.entries(lc3SettingCalls)) {
    f(settings[key as keyof typeof lc3SettingCalls]);
  }
})

// Settings
type SettingKeys = keyof LC3Settings | "all";
function saveSettings(setting: SettingKeys) {
  if (setting === "all") {
    for (let [key, f] of Object.entries(lc3SettingCalls)) {
      f(settings[key as keyof typeof lc3SettingCalls]);
    }
    storage.setAll(settings);
  } else {
    if (setting in lc3SettingCalls) {
      let s = setting as keyof typeof lc3SettingCalls;
      lc3SettingCalls[s](settings[s]);
    }
    storage.set(setting, settings[setting]);
  }
}

// Updater
function updateConfirmed() {
  settings.ignore_update = false;
  saveSettings("ignore_update");
  download_bar.value = true;
  autoUpdater.send("update_confirmed");
}
function downloadUpdate() {
  settings.ignore_update = false;
  update_dialog.value = true;
  saveSettings("ignore_update");
}
function ignoreUpdate() {
  settings.ignore_update = true;
  update_dialog.value = false;
  saveSettings("ignore_update");
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