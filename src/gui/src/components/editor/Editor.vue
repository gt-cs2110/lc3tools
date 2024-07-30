<template>
  <!-- Sidebar -->
  <v-navigation-drawer permanent rail>
    <v-list-item @click="openFile()" prepend-icon="folder_open">
      <v-tooltip location="right" activator="parent" text="Open File" />
    </v-list-item>
    <v-list-item @click="saveFile()">
      <template v-slot:prepend>
        <v-badge color="orange-darken-2" v-model="editorContentChanged">
          <template v-slot:badge>
            <strong>!</strong>
          </template>
          <v-icon icon="save"></v-icon>
        </v-badge>
      </template>

      <v-tooltip location="right" activator="parent" text="Save File" />
    </v-list-item>
    <v-list-item @click="saveFileAs()" prepend-icon="note_add">
      <v-tooltip location="right" activator="parent" text="Save File As" />
    </v-list-item>
    <v-list-item @click="build()" prepend-icon="build">
      <v-tooltip location="right" activator="parent">
        <span v-if="activeFileStore.path === null">Assemble or Convert</span>
        <span v-else-if="activeFileStore.path.endsWith('.asm')">Assemble</span>
        <span v-else-if="activeFileStore.path.endsWith('.bin')">Convert</span>
        <span v-else>Build</span>
      </v-tooltip>
    </v-list-item>
    <v-list-item @click="toggleConsole()" prepend-icon="terminal">
      <v-tooltip location="right" activator="parent" text="Toggle Console" />
    </v-list-item>
  </v-navigation-drawer>
  <!-- Main editor content -->
  <v-main>
    <!-- Don't mind me, just blatantly ignoring Vuetify grid to use flex -->
    <v-container fluid class="fill-height">
      <v-row class="align-self-stretch flex-column" no-gutters>
        <h3 id="filename" class="view-header">{{ filename }}</h3>
        <v-col class="flex-grow-1 flex-shrink-0">
          <v-ace-editor
            id="ace-editor"
            class="elevation-2"
            v-model:value="editor.current_content"
            lang="lc3"
            v-bind:theme="editorTheme"
            ref="aceEditorRef"
          />
        </v-col>
        <v-col class="flex-grow-0 flex-shrink-1" v-if="showConsole">
          <div
            id="console"
            class="elevation-4"
            v-html="consoleStr"
          ></div>
        </v-col>
      </v-row>
    </v-container>
  </v-main>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import API from "../../api";
import { onBeforeRouteUpdate } from "vue-router";
// Editor
import "./ace-cfg";
import ace from "ace-builds";
import { VAceEditor } from "vue3-ace-editor";
import { CreateLc3CompletionProvider } from "./completions";
import Convert from "ansi-to-html";
import { useActiveFileStore } from "../../store/active_file";
import { useSettingsStore } from "../../store/settings";
import { storeToRefs } from "pinia";
import { VAceEditorInstance } from "vue3-ace-editor/types";

declare const api: API;
const { lc3, dialog, fs } = api;

const activeFileStore = useActiveFileStore();
const settings = useSettingsStore();

const editor = ref({
  original_content: "",
  current_content: ""
});
const editorContentChanged = computed(() => editor.value.original_content != editor.value.current_content);
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
  return typeof fp === "string" ? fs.basename(fp) : "Untitled";
})

const aceEditorRef = ref<VAceEditorInstance>();
const aceEditor = computed(() => aceEditorRef.value?.getAceInstance());
// ace editor init:
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
    exec: saveFile
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

// autosave every 5 minutes (cool!)
onMounted(async () => {
  setInterval(autosaveFile, 5 * 60 * 1000);
});

// handle line refs
onBeforeRouteUpdate((to, from) => {
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

      let { Range } = ace.require("ace/range");
      aceEditor.value.gotoLine(slno, scno, true);
      aceEditor.value.getSelection().setRange(new Range(slno, scno, elno, ecno));
    }
  }
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
  // Todo: try catch around this
  let new_file = await dialog.showModal("save", {
    filters: [
      { name: "Assembly", extensions: ["asm"] },
      { name: "Binary", extensions: ["bin"] }
    ]
  });

  if (!new_file.canceled) {
    await _writeFile(new_file.filePath);
  }

  return !new_file.canceled;
}
async function saveFile() {
  // Todo: try catch around this
  // If we don't have a file, create one
  let saveSuccess = true;
  if (activeFileStore.path === null) {
    saveSuccess = await saveFileAs();
  } else {
    await _writeFile(activeFileStore.path);
  }

  if (saveSuccess) {
    await build();
  }
}
async function autosaveFile() {
  if (activeFileStore.path !== null && editorContentChanged.value) {
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
        { name: "Assembly", extensions: ["asm"] },
        { name: "Binary", extensions: ["bin"] }
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
  } else if (activeFileStore.path.endsWith(".bin")) {
    try {
      lc3.convertBin(activeFileStore.path);
    } catch (e) {
      success = false;
    }
    output = lc3.getAndClearOutput();
  } else if (activeFileStore.path.endsWith(".asm")) {
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

  // VS Code's Dark+ terminal colors.
  let convert = new Convert({
    colors: [
    "#000000", "#CD3131", "#0DBC79", "#E5E510", 
    "#2472C8", "#BC3FBC", "#11A8CD", "#E5E5E5", 
    "#666666", "#F14C4C", "#23D18B", "#F5F543", 
    "#3B8EEA", "#D670D6", "#29B8DB", "#E5E5E5"
    ]
  });
  consoleStr.value = convert.toHtml(output);
  
  if (success) {
    activeFileStore.touchBuildTime();
  }
}
</script>

<style>
.ace-twilight .ace_marker-layer .ace_selection {
  background: rgb(60, 97, 146) !important;
}
</style>

<style scoped>
#filename {
  text-align: center;
}

#ace-editor {
  overflow: hidden;
  justify-self: center;
  height: 100%;
}

#console {
  overflow: auto;
  font-family: Consolas, Menlo, Courier, monospace;
  margin: 15px 0 5px 0;
  padding: 10px;
  white-space: pre-wrap;
  height: 170px;
}
</style>
