<script setup lang="ts">
import { useActiveFileStore } from "../../store/active_file";
import { useSettingsStore } from "../../store/settings";
// Vue stuff
import { computed, onMounted, ref, useTemplateRef, watch } from "vue";
import "vuetify/components";
import { storeToRefs } from "pinia";
// Editor
import "./ace-cfg";
import ace from "ace-builds";
import { VAceEditor } from "vue3-ace-editor";
import type { VAceEditorInstance } from "vue3-ace-editor/types";
import { CreateLc3CompletionProvider } from "./completions";
//
import Console from "../Console.vue";
import { mdiConsole, mdiContentSave, mdiContentSaveEdit, mdiFolderOpen, mdiLinkVariant, mdiWrench } from "@mdi/js";

const { lc3, dialog, fs } = window.api;
const activeFileStore = useActiveFileStore();
const settings = useSettingsStore();

const editor = ref({
  original_content: "",
  current_content: ""
});
const editorContentChanged = computed(() => editor.value.original_content != editor.value.current_content);
const editorIsEmpty = computed(() => editor.value.original_content === "" && editor.value.current_content === "");
const consoleStr = ref("");
const showConsole = ref(false);

const settingsRefs = storeToRefs(settings);
const editorBinding = settingsRefs.editor_binding;
const tabSize = computed<number>(() => {
  return settingsRefs.soft_tabs.value ? settingsRefs.soft_tab_size.value : -1
});
const autocompleteMode = settingsRefs.autocomplete;
const editorTheme = computed(() => ({
  "light": "textmate",
  "dark": "twilight"
})[settingsRefs.theme.value]);
const filename = computed(() => {
  const fp = activeFileStore.path;
  return typeof fp === "string" ? fs.basename(fp) : "\u{200B}";
})

const aceEditorRef = useTemplateRef<VAceEditorInstance>("aceEditorRef");
const aceEditor = computed(() => aceEditorRef.value?.getAceInstance());

// ace editor setup:
watch(aceEditor, (editor) => {
  editor.setShowPrintMargin(false);
  editor.setOptions({
    fontSize: "1.25em",
    scrollPastEnd: 0.7,
    enableBasicAutocompletion: [
      CreateLc3CompletionProvider(() => autocompleteMode.value)
    ],
    enableLiveAutocompletion: true
  });
  editor.commands.addCommands([
    {
      name: "save",
      bindKey: { win: "Ctrl-S", mac: "Cmd-S" },
      exec: () => saveFileThen(build)
    },
    {
      name: "build",
      bindKey: { win: "Ctrl-Enter", mac: "Cmd-Enter" },
      exec: build
    },
    {
      name: "open",
      bindKey: { win: "Ctrl-O", mac: "Cmd-O" },
      exec: (e, path) => openFile(path)
    }
  ]);

  // Vim custom config:
  ace.config.loadModule("ace/keyboard/vim", module => {
    const VimApi = module.CodeMirror.Vim;
    VimApi.defineEx("write", "w", function(cm: any, input: any) {
      cm.ace.execCommand("save");
    });
  });

  // Initialize editor settings:
  setEditorBinding(settingsRefs.editor_binding.value);
  setTabSize(tabSize.value);
}, { once: true });

// On editor binding update:
watch(editorBinding, setEditorBinding);
watch(tabSize, setTabSize);

onMounted(() => {
  // autosave every 5 minutes (cool!)
  setInterval(autosaveFile, 5 * 60 * 1000);
});

// Methods
function toggleConsole() {
  showConsole.value = !showConsole.value;
}
function setEditorBinding(binding: typeof settings["editor_binding"]) {
  if (typeof aceEditor.value === "undefined") {
    console.warn("Ace editor did not exist when trying to set keyboard binding");
    return;
  }

  if (binding === "vim") {
    aceEditor.value.setKeyboardHandler("ace/keyboard/vim");
  } else {
    aceEditor.value.setKeyboardHandler("");
  }
}
function setTabSize(binding: typeof tabSize.value) {
  aceEditor.value.setOptions({
    useSoftTabs: binding > 0,
    tabSize: Math.max(binding, 1)
  });
}

async function link() {
  const inputs = await dialog.showModal("open", {
    properties: ["openFile", "multiSelections"],
    filters: [
      { name: "Object Files", extensions: ["obj"] }
    ]
  });
  if (!inputs.canceled) {
    const output = await dialog.showModal("save", {
      defaultPath: "linked.obj",
      filters: [
        { name: "Object Files", extensions: ["obj"] }
      ]
    })

    try {
      lc3.link(inputs.filePaths, output.filePath);
    } catch {
      // Don't crash on link failure.
    }
    
    showConsole.value = true;
    consoleStr.value = lc3.getAndClearOutput();
  }
}

async function _writeFile(fp: string, content: string | undefined = undefined) {
  if (typeof content === "undefined") content = editor.value.current_content;

  await fs.write(fp, content);
  editor.value.original_content = content;
  activeFileStore.path = fp;
}
async function saveFileAs() {
  const new_file = await dialog.showModal("save", {
    filters: [
      { name: "Assembly", extensions: ["asm"] }
    ]
  });

  if (!new_file.canceled) {
    await _writeFile(new_file.filePath);
  }

  return !new_file.canceled;
}
async function saveFile() {
  // If we don't have a file, create one
  let saveSuccess = true;
  if (activeFileStore.path === null) {
    saveSuccess = editorIsEmpty.value || await saveFileAs();
  } else {
    await _writeFile(activeFileStore.path);
  }

  return saveSuccess;
}
// Save the current file, then do something secondary (if saving was successful).
async function saveFileThen(f: () => void | Promise<void>) {
  const success = await saveFile();
  if (success) await f();
}

async function autosaveFile() {
  if (activeFileStore.path !== null && editorContentChanged.value && !editorIsEmpty.value) {
    await _writeFile(activeFileStore.path);
  }
}
async function openFile(path: string | undefined = undefined) {
  // Only allow open if accept on unsaved changes:
  const accept = await triggerUnsavedChangesModal();
  if (!accept) return;

  // if not given a path, open a dialog to ask user for file
  let selected_files: string[] = [];
  if (typeof path !== "string") {
    const result = await dialog.showModal("open", {
      properties: ["openFile"],
      filters: [
        { name: "Assembly", extensions: ["asm"] }
      ]
    });

    selected_files = result.filePaths;
  } else {
    selected_files = [path];
  }

  // Dialog returns an array of files, we only care about the first one
  if (selected_files.length > 0) {
    const active_file = selected_files[0];
    editor.value.original_content = editor.value.current_content = await fs.read(active_file);
    activeFileStore.path = active_file;
  }
}
async function dropFile(e: DragEvent) {
  const file = e.dataTransfer.files[0];
  if (file?.name.toLowerCase().endsWith("asm")) {
    const accept = await triggerUnsavedChangesModal();
    if (accept) {
      await openFile(fs.getPath(file));
    }
  }
}

/**
 * Opens a modal prompting the user to save changes (if they have unsaved changes).
 * This may not open a modal if there are no changes to save.
 * 
 * @returns whether the action following this modal was not canceled, i.e.,
 * - This method returns true if no modal was required or if `Yes` or `No` were pressed
 * - This method returns false if `Cancel` was pressed
 */
async function triggerUnsavedChangesModal() {
  if (!editorContentChanged.value) return true;

  const buttons = ["Yes", "No", "Cancel"];
  const cancelId = 2;

  // Save warning
  const clicked = await dialog.showModal("box", {
    type: 'warning',
    title: 'Confirm',
    message: `You have unsaved changes to ${filename.value}. Would you like to save your changes?`,
    buttons,
    cancelId
  });

  if (clicked.response === 0) {
    await saveFile();
  }
  return clicked.response !== cancelId;
}

async function build() {
  // save the file if it hasn't been saved
  if (editorContentChanged.value) {
    await saveFile();
  }

  // show console when assembling
  showConsole.value = true;
  let success = true;
  let output = "";
  if (activeFileStore.path === null) {
    success = false;
    output = "No file to build";
  } else if (activeFileStore.path.toLowerCase().endsWith(".asm")) {
    try {
      lc3.assemble(activeFileStore.path);
    } catch {
      success = false;
    }
    output = lc3.getAndClearOutput();
  } else {
    success = false;
    output = `Cannot build file ${activeFileStore.path}`;
  }
  consoleStr.value = output;
  
  if (success) {
    activeFileStore.touchBuildTime();
  }
}

defineExpose({ ace, aceEditor });
</script>

<script lang="ts">
// Necessary because beforeRouteEnter doesn't exist in Composition API form
// (as of Vue 3.4.34, Vue Router 4.4.0)
export default {
  beforeRouteEnter(to, from, next) {
    next((vm: any) => {
      if (to.hash) {
        // format L999C999-L999C999
        const hash_pattern = /^#?L(\d+)C(\d+)-L(\d+)C(\d+)$/;
        const match = to.hash.match(hash_pattern);
        if (match) {
          const [_, slno_str, scno_str, elno_str, ecno_str] = match;
          const slno = parseInt(slno_str, 10);
          const scno = parseInt(scno_str, 10);
          const elno = parseInt(elno_str, 10);
          const ecno = parseInt(ecno_str, 10);
  
          const { Range } = vm.ace.require("ace/range");
          vm.aceEditor.gotoLine(slno, scno, true);
          vm.aceEditor.getSelection().setRange(new Range(slno, scno, elno, ecno));
        }
      }
    })
  }
}
</script>

<template>
  <div class="contents">
    <!-- Sidebar -->
    <v-navigation-drawer
      permanent
      rail
    >
      <v-list-item
        :prepend-icon="mdiFolderOpen"
        @click="openFile()"
      >
        <v-tooltip
          location="right"
          activator="parent"
          text="Open File"
        />
      </v-list-item>
      <v-list-item @click="saveFileThen(build)">
        <template #prepend>
          <v-badge
            v-model="editorContentChanged"
            color="orange-darken-2"
          >
            <template #badge>
              <strong>!</strong>
            </template>
            <v-icon :icon="mdiContentSave" />
          </v-badge>
        </template>

        <v-tooltip
          location="right"
          activator="parent"
          text="Save File"
        />
      </v-list-item>
      <v-list-item
        :prepend-icon="mdiContentSaveEdit"
        @click="saveFileAs()"
      >
        <v-tooltip
          location="right"
          activator="parent"
          text="Save File As"
        />
      </v-list-item>
      <v-list-item
        :prepend-icon="mdiWrench"
        @click="build()"
      >
        <v-tooltip
          location="right"
          activator="parent"
        >
          <span>Assemble</span>
        </v-tooltip>
      </v-list-item>
      <v-list-item
        :prepend-icon="mdiConsole"
        @click="toggleConsole()"
      >
        <v-tooltip
          location="right"
          activator="parent"
          text="Toggle Console"
        />
      </v-list-item>
      <v-list-item
        :prepend-icon="mdiLinkVariant"
        @click="link()"
      >
        <v-tooltip
          location="right"
          activator="parent"
          text="Link Object Files"
        />
      </v-list-item>
    </v-navigation-drawer>
    <!-- Main editor content -->
    <v-main>
      <!-- Don't mind me, just blatantly ignoring Vuetify grid to use flex -->
      <v-container
        fluid
        class="fill-height"
      >
        <v-row
          class="align-self-stretch flex-column"
          no-gutters
        >
          <h3 class="view-header">
            {{ filename }}
          </h3>
          <v-col class="d-flex flex-grow-1 flex-shrink-0">
            <v-ace-editor
              id="ace-editor"
              ref="aceEditorRef"
              v-model:value="editor.current_content"
              class="flex-grow-1 elevation-2"
              lang="lc3"
              :theme="editorTheme"
              @drop.prevent="dropFile"
              @dragover.prevent
            />
          </v-col>
          <v-col
            v-if="showConsole"
            class="flex-grow-0 flex-shrink-1"
          >
            <console 
              id="console"
              v-model="consoleStr"
              float="top"
            />
          </v-col>
        </v-row>
      </v-container>
    </v-main>
  </div>
</template>

<style>
.ace-twilight .ace_marker-layer .ace_selection {
  background: rgb(60, 97, 146) !important;
}
</style>

<style scoped>
.view-header {
  text-align: center;
  padding-bottom: 5px;
}

#ace-editor {
  overflow: hidden;
  justify-self: center;
  height: 100%;
}

#console {
  margin: 15px 0 5px 0;
  height: 170px;
}
</style>
