<script setup lang="ts">
import { onMounted, useTemplateRef, watch } from "vue";
import { LC3Settings, useSettingsStore } from "./store/settings";
import { storeToRefs } from "pinia";
import { useRoute } from "vue-router";

const settingsPopover = useTemplateRef("settingsPopover");
const { lc3, storage } = window.api;

const route = useRoute();
// Settings
const settings = useSettingsStore();
settings.$patch(storage.getAll());

const settingsRefs = storeToRefs(settings);
// Store settings to persistent storage when any modifications occur.
for (const [key, r] of Object.entries(settingsRefs)) {
  watch(r, () => {
    saveSettings(key as keyof LC3Settings);
  });
}
// Apply theme to page:
watch(settingsRefs.theme, theme => {
  document.documentElement.classList.remove("light", "dark");
  document.documentElement.classList.add(theme);
}, { immediate: true });

const selectButtons = {
  theme: [
    {value: 'light', label: 'Light'},
    {value: 'dark', label: 'Dark'}
  ],
  editor_binding: [
    {value: 'standard', label: 'Standard'},
    {value: 'vim', label: 'Vim'}
  ],
  autocomplete: [
    {value: 'none', label: 'None'},
    {value: 'basic', label: 'Basic'},
    {value: 'full', label: 'Full'}
  ],
  numbers: [
    {value: 'signed', label: 'Signed'},
    {value: 'unsigned', label: 'Unsigned'}
  ]
} satisfies Partial<{ [K in keyof LC3Settings]: { value: LC3Settings[K], label: string }[] }>;
// Any calls that need to occur to the LC3 engine when a property is updated.
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

<template>
  <div class="flex flex-col h-screen">
    <Menubar>
      <template #start>
        <div class="flex items-center gap-1">
          <h1 class="text-xl">
            <strong>LC3</strong>Tools
          </h1>
          <Button
            icon="pi"
            variant="text"
            rounded
            severity="secondary"
            aria-label="Settings"
            @click="settingsPopover?.toggle"
          >
            <MdiCog />
          </Button>
        </div>
      </template>
      <template #end>
        <Tabs :value="route.name as string">
          <TabList>
            <Tab
              v-tooltip.bottom="'Editor'"
              value="editor"
              to="/editor"
              as="router-link"
              class="px-6 py-1"
              aria-label="Editor"
            >
              <MdiCodeTags
                width="2em"
                height="2em"
              />
            </Tab>
            <Tab
              v-tooltip.bottom="'Simulator'"
              value="simulator"
              to="/simulator"
              as="router-link"
              class="px-6 py-1"
              aria-label="Simulator"
            >
              <MdiMemory
                width="2em"
                height="2em"
              />
            </Tab>
          </TabList>
        </Tabs>
      </template>
    </Menubar>
  
    <router-view
      v-slot="{ Component }"
      class="flex grow overflow-auto"
    >
      <keep-alive>
        <component :is="Component" />
      </keep-alive>
    </router-view>
  </div>

  <Popover ref="settingsPopover">
    <div class="popover-menu">
      <div>
        <label>
          <span>Theme</span>
          <SelectButton
            v-model="settings.theme"
            :options="selectButtons.theme"
            option-label="label"
            option-value="value"
            :allow-empty="false"
          />
        </label>
        <label>
          <span>Editor Key Bindings</span>
          <SelectButton
            v-model="settings.editor_binding"
            :options="selectButtons.editor_binding"
            option-label="label"
            option-value="value"
            :allow-empty="false"
          />
        </label>
        <label>
          <span>Autocomplete</span>
          <SelectButton
            v-model="settings.autocomplete"
            :options="selectButtons.autocomplete"
            option-label="label"
            option-value="value"
            :allow-empty="false"
          />
        </label>
        <label>
          <span>Soft Tabs</span>
          <div class="flex items-center gap-3">
            <Checkbox
              v-model="settings.soft_tabs"
              binary
            />
            <InputNumber
              v-model="settings.soft_tab_size"
              :use-grouping="false"
              :disabled="!settings.soft_tabs"
              :min="0"
              input-class="w-24"
            />
          </div>
        </label>
      </div>
      <Divider />
      <div>
        <label>
          <span>Number View</span>
          <SelectButton
            v-model="settings.numbers"
            :options="selectButtons.numbers"
            option-label="label"
            option-value="value"
            :allow-empty="false"
          />
        </label>
        <label>
          <span>Pause on HALT and exceptions</span>
          <ToggleSwitch v-model="settings.pause_on_fatal_trap" />
        </label>
        <label>
          <span>Clear output on object file reload</span>
          <ToggleSwitch v-model="settings.clear_out_on_reload" />
        </label>
        <label>
          <div class="flex gap-1">
            <span>Ignore privileged mode</span>
            <MdiAlert
              v-tooltip="settings.ignore_privilege ? 'This setting may result in behavior inconsistent with the autograder' : ''"
              class="text-red-500 inline-block transition-opacity"
              :class="{ 'opacity-0': !settings.ignore_privilege }"
            />
          </div>
          <ToggleSwitch v-model="settings.ignore_privilege" />
        </label>
        <label>
          <span>Reduce flashing in simulator</span>
          <ToggleSwitch v-model="settings.reduce_flashing" />
        </label>
      </div>
      <Divider />
      <div>Issues? Post on CS 2110 Piazza!</div>
    </div>
  </Popover>
</template>

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
}

/* https://stackoverflow.com/q/56973002/11984788 */
html {
    overflow-y: auto !important;
}
</style>

<style scoped>
@reference "@/style.css";

.popover-menu > div {
  @apply flex flex-col gap-3;
}
.popover-menu > div > label {
  @apply flex justify-between items-center gap-2;
}
</style>