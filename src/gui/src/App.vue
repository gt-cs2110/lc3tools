<template>
  <div id="app">
    <v-app id="lc3tools" v-bind:theme="settings.theme">
      <v-app-bar density="compact" :elevation="2">
        <v-app-bar-title>
          <strong>LC3</strong>Tools
          <!-- Put buttons next to title -->
          <v-btn
            icon
            flat
            @click="downloadUpdate"
            v-if="update_available"
          >
            <v-icon color="green" icon="info"></v-icon>
            <v-tooltip location="bottom" activator="parent">
              <span>Update</span>
            </v-tooltip>
          </v-btn>
          <v-btn icon flat>
            <v-icon icon="settings"></v-icon>
            <v-menu activator="parent" :close-on-content-click="false">
              <v-card>
                <v-container>
                  <!-- Should use v-row, v-col, but those are grid not flex -->
                  <div class="d-flex justify-space-between">
                    <h3 class="flex-grow-1">Theme</h3>
                    <v-radio-group
                      class="flex-shrink-1"
                      color="primary"
                      @change="saveSettings('theme')"
                      v-model="settings.theme"
                      inline
                    >
                      <v-spacer></v-spacer>
                      <v-radio label="Light" value="light"></v-radio>
                      <v-radio label="Dark" value="dark"></v-radio>
                      </v-radio-group>
                  </div>
                  <div class="d-flex justify-space-between">
                    <h3 class="flex-grow-1">Number View</h3>
                    <v-radio-group
                      class="flex-shrink-1"
                      color="primary"
                      @change="saveSettings('numbers')"
                      v-model="settings.numbers"
                      inline
                    >
                      <v-spacer></v-spacer>
                      <v-radio label="Unsigned" value="unsigned"></v-radio>
                      <v-radio label="Signed" value="signed"></v-radio>
                    </v-radio-group>
                  </div>
                  <div class="d-flex justify-space-between">
                    <h3 class="flex-grow-1">Editor Key Binding</h3>
                    <v-radio-group
                      class="flex-shrink-1"
                      color="primary"
                      @change="saveSettings('editor_binding')"
                      v-model="settings.editor_binding"
                      inline
                    >
                      <v-spacer></v-spacer>
                      <v-radio label="Standard" value="standard"></v-radio>
                      <v-radio label="Vim" value="vim"></v-radio>
                    </v-radio-group>
                  </div>
                  <div class="d-flex justify-space-between">
                    <h3 class="flex-grow-1">Autocomplete</h3>
                    <v-radio-group
                      class="flex-shrink-1"
                      color="primary"
                      @change="saveSettings('autocomplete')"
                      v-model="settings.autocomplete"
                      inline
                    >
                      <v-spacer></v-spacer>
                      <v-radio label="None" value="none"></v-radio>
                      <v-radio label="Basic" value="basic"></v-radio>
                      <v-radio label="Full" value="full"></v-radio>
                    </v-radio-group>
                  </div>
                  <div class="d-flex justify-space-between">
                    <h3 class="flex-grow-1">Stop execution on reaching HALT</h3>
                    <v-switch
                      class="flex-shrink-1"
                      color="primary"
                      @change="saveSettings('run_until_halt')"
                      v-model="settings.run_until_halt"
                    >
                    </v-switch>
                  </div>
                  <div class="d-flex justify-space-between">
                    <h3 class="flex-grow-1">Clear output on object file reload</h3>
                    <v-switch
                      class="flex-shrink-1"
                      color="primary"
                      @change="saveSettings('clear_out_on_reload')"
                      v-model="settings.clear_out_on_reload"
                    >
                    </v-switch>
                  </div>
                  <div class="d-flex justify-space-between">
                    <div class="flex-grow-1">
                      <h3>Ignore privileged mode</h3>
                      <p class="text-red" v-if="settings.ignore_privilege">
                        May result in inconsistency with the grader.
                      </p>
                    </div>
                    <v-switch
                      class="flex-shrink-1"
                      color="primary"
                      @change="saveSettings('ignore_privilege')"
                      v-model="settings.ignore_privilege"
                    >
                    </v-switch>
                  </div>
                  <div class="d-flex justify-space-between">
                    <div class="flex-grow-1">
                      <h3>Use less strict assembly</h3>
                      <p class="text-red" v-if="settings.liberal_asm">
                        May result in inconsistency with the grader.
                      </p>
                    </div>
                    <v-switch
                      class="flex-shrink-1"
                      color="primary"
                      @change="saveSettings('liberal_asm')"
                      v-model="settings.liberal_asm"
                    >
                    </v-switch>
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
          <v-tab exact to="/editor" icon>
            <v-icon size="x-large" icon="code"></v-icon>
            <v-tooltip location="bottom" activator="parent">
              <span>Editor</span>
            </v-tooltip>
          </v-tab>
          <v-tab exact to="/simulator" icon>
            <v-icon size="x-large" icon="memory"></v-icon>
            <v-tooltip location="bottom" activator="parent">
              <span>Simulator</span>
            </v-tooltip>
          </v-tab>
        </v-tabs>
      </v-app-bar>

      <router-view v-slot="{ Component }">
        <keep-alive>
          <component :is="Component" />
        </keep-alive>
      </router-view>

      <v-dialog v-model="update_dialog" max-width="400" persistent>
        <v-card>
          <v-card-title v-if="!download_bar" class="headline"
            >Update Available</v-card-title
          >

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
              v-bind:modelValue="
                (update.download_transferred / update.download_size) * 100
              "
            ></v-progress-linear>
          </v-card-text>

          <v-card-actions v-if="!download_bar">
            <v-btn
              icon
              flat
              @click="ignoreUpdate"
            >
              <v-icon icon="delete"></v-icon>
              <v-tooltip location="top" activator="parent">
                <span>Ignore</span>
              </v-tooltip>
            </v-btn>

            <v-btn
              icon
              flat
              @click="update_dialog = false"
            >
              <v-icon icon="thumb_down" color="red-darken-1"></v-icon>
              <v-tooltip location="top" activator="parent">
                <span>No</span>
              </v-tooltip>
            </v-btn>

            <v-btn
              icon
              flat
              @click="updateConfirmed"
            >
              <v-icon icon="thumb_up" color="green-darken-1"></v-icon>
              <v-tooltip location="top" activator="parent">
                <span>Yes</span>
              </v-tooltip>
            </v-btn>
          </v-card-actions>
        </v-card>
      </v-dialog>
    </v-app>
  </div>
</template>
  
<script setup lang="ts">
import { onMounted, ref } from "vue";
import API from "./api";
declare const api: API;
const { lc3, autoUpdater, storage } = api;

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
const settings = ref({
  theme: "light",
  numbers: "signed",
  editor_binding: "standard",
  ignore_privilege: false,
  liberal_asm: false,
  ignore_update: false,
  run_until_halt: true,
  clear_out_on_reload: true,
  autocomplete: "full"
});

onMounted(() => {
  autoUpdater.on((message, progress) => {
    if (message === "update_available") {
        // Show the settings modal
        update_dialog.value = !settings.value.ignore_update;
        update_available.value = true;
      }
      if (message === "download_progress") {
        update.value.download_speed = progress.bytesPerSecond;
        update.value.download_size = progress.total;
        update.value.download_transferred = progress.transferred;
      }
  })
})

// Settings
type Setting = keyof typeof settings.value | "all";
settings.value = {
  ...settings.value,
  ...storage.getAll() as typeof settings,
  liberal_asm: false
};
function saveSettings(setting: Setting) {
  let settings_ = settings.value;
  if (setting === "all") {
    lc3.setIgnorePrivilege(settings_.ignore_privilege);
    lc3.setEnableLiberalAsm(settings_.liberal_asm);
    lc3.setRunUntilHalt(settings_.run_until_halt);
    storage.setAll(settings_);
  } else {
    if (setting === "ignore_privilege") lc3.setIgnorePrivilege(settings_.ignore_privilege);
    if (setting === "liberal_asm") lc3.setEnableLiberalAsm(settings_.liberal_asm);
    if (setting === "run_until_halt") lc3.setRunUntilHalt(settings_.run_until_halt);
    storage.set(setting, settings_[setting]);
  }
}

// Updater
function updateConfirmed() {
  settings.value.ignore_update = false;
  saveSettings("ignore_update");
  download_bar.value = true;
  autoUpdater.send("update_confirmed");
}
function downloadUpdate() {
  settings.value.ignore_update = false;
  update_dialog.value = true;
  saveSettings("ignore_update");
}
function ignoreUpdate() {
  settings.value.ignore_update = true;
  update_dialog.value = false;
  saveSettings("ignore_update");
}
</script>

<style lang="css">
@import "index.css";
</style>