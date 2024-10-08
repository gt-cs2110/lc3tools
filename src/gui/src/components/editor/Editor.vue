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
          <span v-if="activeFileStore.path === null">Assemble</span>
          <span v-else>Build</span>
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
          <v-col class="flex-grow-1 flex-shrink-0">
            <v-ace-editor
              id="ace-editor"
              ref="aceEditorRef"
              v-model:value="editor.current_content"
              class="elevation-2"
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
import { mdiConsole, mdiContentSave, mdiContentSaveEdit, mdiFolderOpen, mdiWrench } from "@mdi/js";

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
const autocompleteMode = settingsRefs.autocomplete;
const editorTheme = computed(() => ({
  "light": "textmate",
  "dark": "twilight"
})[settingsRefs.theme.value]);
const filename = computed(() => {
  let fp = activeFileStore.path;
  return typeof fp === "string" ? fs.basename(fp) : "\u{200B}";
})

const aceEditorRef = useTemplateRef<VAceEditorInstance>("aceEditorRef");
const aceEditor = computed(() => aceEditorRef.value?.getAceInstance());

// ace editor setup:
watch(aceEditorRef, (ref) => {
  let aceEditor = ref.getAceInstance();

  aceEditor.setShowPrintMargin(false);
  aceEditor.setOptions({
    fontSize: "1.25em",
    scrollPastEnd: 0.7
  });
  aceEditor.setOptions({
    enableBasicAutocompletion: [
      CreateLc3CompletionProvider(() => autocompleteMode.value)
    ],
    enableLiveAutocompletion: true
  });
  aceEditor.commands.addCommand({
    name: "save",
    bindKey: { win: "Ctrl-S", mac: "Cmd-S" },
    exec: () => saveFileThen(build)
  });
  aceEditor.commands.addCommand({
    name: "build",
    bindKey: { win: "Ctrl-Enter", mac: "Cmd-Enter" },
    exec: build
  });
  aceEditor.commands.addCommand({
    name: "open",
    bindKey: { win: "Ctrl-O", mac: "Cmd-O" },
    exec: (e, path) => openFile(path)
  });
}, { once: true });

// On editor binding update:
watch(editorBinding, binding => {
  if (binding === "vim") {
    aceEditor.value.setKeyboardHandler("ace/keyboard/vim");
    ace.config.loadModule("ace/keyboard/vim", module => {
      let VimApi = module.CodeMirror.Vim;
      VimApi.defineEx("write", "w", function(cm: any, input: any) {
        cm.ace.execCommand("save");
      });
    })
  } else {
    aceEditor.value.setKeyboardHandler("");
  }
})

onMounted(() => {
  // autosave every 5 minutes (cool!)
  setInterval(autosaveFile, 5 * 60 * 1000);
});

// Methods
function toggleConsole() {
  showConsole.value = !showConsole.value;
}
async function _writeFile(fp: string, content: string | undefined = undefined) {
  if (typeof content === "undefined") content = editor.value.current_content;

  await fs.write(fp, content);
  editor.value.original_content = content;
  activeFileStore.path = fp;
}
async function saveFileAs() {
  let new_file = await dialog.showModal("save", {
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
async function saveFileThen(f: () => void) {
  let success = await saveFile();
  if (success) f();
}

async function autosaveFile() {
  if (activeFileStore.path !== null && editorContentChanged.value && !editorIsEmpty.value) {
    await _writeFile(activeFileStore.path);
  }
}
async function openFile(path: string | undefined = undefined) {
  // Todo: try catch around this
  // if not given a path, open a dialog to ask user for file
  let selected_files: string[] = [];
  if (typeof path !== "string") {
    let result = await dialog.showModal("open", {
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
    let active_file = selected_files[0];
    editor.value.original_content = editor.value.current_content = await fs.read(active_file);
    activeFileStore.path = active_file;
  }
}
async function dropFile(e: DragEvent) {
  let file = e.dataTransfer.files[0];
  if (file?.name.toLowerCase().endsWith("asm")) {
    if (editorContentChanged.value) {
      const buttons = ["Yes", "No", "Cancel"]
      // Save warning
      let clicked = await dialog.showModal("box", {
        type: 'warning',
        title: 'Confirm',
        message: `You have unsaved changes to ${filename.value}. Would you like to save your changes?`,
        buttons,
        cancelId: 2
      });

      let response = buttons[clicked.response];
      if (response === "Yes") await saveFileThen(() => openFile(fs.getPath(file)));
      else if (response === "No") await openFile(fs.getPath(file));
    } else {
      await saveFileThen(() => openFile(fs.getPath(file)));
    }
  }
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
    } catch (e) {
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
        let hash_pattern = /^#?L(\d+)C(\d+)-L(\d+)C(\d+)$/;
        let match = to.hash.match(hash_pattern);
        if (match) {
          let [_, slno_str, scno_str, elno_str, ecno_str] = match;
          let slno = parseInt(slno_str, 10);
          let scno = parseInt(scno_str, 10);
          let elno = parseInt(elno_str, 10);
          let ecno = parseInt(ecno_str, 10);
  
          let { Range } = vm.ace.require("ace/range");
          vm.aceEditor.gotoLine(slno, scno, true);
          vm.aceEditor.getSelection().setRange(new Range(slno, scno, elno, ecno));
        }
      }
    })
  }
}
</script>

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

.contents {
  display: contents;
}
</style>