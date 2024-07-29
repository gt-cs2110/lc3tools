<template>
  <!-- Main editor content -->
  <v-main>
    <v-container fluid class="fill-height">
      <v-row>
        <v-col>
          <h3 id="filename" class="view-header">filename</h3>
          <v-ace-editor
            id="ace-editor"
            class="elevation-2"
            v-model:value="editor.current_content"
            lang="lc3"
            v-bind:theme="darkMode ? 'twilight' : 'textmate'"
            @init="editorInit"
            style="height: 500px"
          />
          <div
            :class="{ 'hide-console': !show_console }"
            id="console"
            class="elevation-4"
            v-html="console_str"
          ></div>
        </v-col>
      </v-row>
    </v-container>
  </v-main>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import API from "../../api";
import { onBeforeRouteUpdate } from "vue-router";
// Editor
import "./ace-cfg";
import ace from "ace-builds";
import { type Ace } from "ace-builds";
import { VAceEditor } from "vue3-ace-editor";
import { CreateLc3CompletionProvider } from "./completions";
import Convert from "ansi-to-html";

declare const api: API;
const { lc3, dialog } = api;

const darkMode = true;
const editor = ref({
  original_content: "",
  current_content: "",
  content_changed: false,
  // for whatever reason, i can't get template ref to cooperate
  // so i'm just using @init to access
  ref: null as Ace.Editor | null
});
const console_str = ref("");
const show_console = ref(false);

// autosave every 5 minutes (cool!)
onMounted(async () => {
  setInterval(autosaveFile, 5 * 60 * 1000)
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
      editor.value.ref.gotoLine(slno, scno, true);
      editor.value.ref.getSelection().setRange(new Range(slno, scno, elno, ecno));
    }
  }
});

function toggleConsole() {
  show_console.value = !show_console.value;
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
    fs.writeFileSync(new_file.filePath, editor.current_content);
    openFile(new_file.filePath);
  }
}
async function saveFile() {
  // Todo: try catch around this
  // If we don't have a file, create one
  if (this.$store.getters.activeFilePath === null) {
    await saveFileAs();
  } else {
    fs.writeFileSync(
      this.$store.getters.activeFilePath,
      this.editor.current_content
    );
    this.editor.original_content = this.editor.current_content;
  }
  this.build();
}
function autosaveFile() {
  if (
    this.$store.getters.activeFilePath !== null &&
    this.editor.original_content !== this.editor.current_content
  ) {
    fs.writeFileSync(
      this.$store.getters.activeFilePath,
      this.editor.current_content
    );
    this.editor.original_content = this.editor.current_content;
  }
}
async function openFile(path: string | undefined) {
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
    this.editor.original_content = this.editor.current_content = fs.readFileSync(
      active_file,
      "utf-8"
    );
    this.$store.commit("setActiveFilePath", active_file);
  }
}
async function build() {
  // save the file if it hasn't been saved
  if (this.editor.content_changed) {
    this.editor.content_changed = false;
    await saveFile();
  }

  // show console when assembling
  show_console.value = true;
  let success = true;
  if (this.$store.getters.activeFilePath.endsWith(".bin")) {
    try {
      lc3.convertBin(this.$store.getters.activeFilePath);
    } catch (e) {
      success = false;
    }
  } else {
    try {
      lc3.assemble(this.$store.getters.activeFilePath);
    } catch (e) {
      success = false;
    }
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
  const temp_console_string = lc3.getAndClearOutput();

  this.console_str = "";
  setTimeout(() => {
    this.console_str = convert.toHtml(temp_console_string);
  }, 200);
  if (success) {
    this.$store.commit("touchActiveFileBuildTime");
  }
}
function editorInit(editor: Ace.Editor) {
  editor.setShowPrintMargin(false);
  editor.setOptions({
    fontSize: "1.25em",
    scrollPastEnd: 0.7
  });
  editor.setOptions({
    enableBasicAutocompletion: [
      CreateLc3CompletionProvider(() => this.autocompleteMode)
    ],
    enableLiveAutocompletion: true
  });
  editor.commands.addCommand({
    name: "save",
    bindKey: { win: "Ctrl-S", mac: "Cmd-S" },
    exec: saveFile
  });
  editor.commands.addCommand({
    name: "build",
    bindKey: { win: "Ctrl-Enter", mac: "Cmd-Enter" },
    exec: build
  });
  editor.commands.addCommand({
    name: "open",
    bindKey: { win: "Ctrl-O", mac: "Cmd-O" },
    exec: (e, path) => openFile(path)
  });
}
</script>


<!-- <script>

export default {
  computed: {
    getFilename() {
      return this.$store.getters.activeFilePath === null
        ? "Untitled"
        : path.basename(this.$store.getters.activeFilePath);
    },
    darkMode() {
      return this.$store.getters.theme === "dark";
    },
    editorBinding() {
      return this.$store.getters.editor_binding;
    },
    autocompleteMode() {
      return this.$store.getters.autocomplete;
    }
  },
  watch: {
    "editor.current_content": function(newContent) {
      // Compare against original content to see if it's changed
      if (newContent !== this.editor.original_content) {
        this.editor.content_changed = true;
      } else {
        this.editor.content_changed = false;
      }
    },
    "editor.original_content": function(newContent) {
      // Compare against original content to see if it's changed
      if (newContent !== this.editor.original_content) {
        this.editor.content_changed = true;
      } else {
        this.editor.content_changed = false;
      }
    },
    editorBinding: function(binding) {
      if (binding === "vim") {
        this.$refs.aceEditor.editor.setKeyboardHandler("ace/keyboard/vim");
        ace.config.loadModule("ace/keyboard/vim", function(module) {
          var VimApi = module.CodeMirror.Vim;
          VimApi.defineEx("write", "w", function(cm, input) {
            cm.ace.execCommand("save");
          });
        });
      } else {
        this.$refs.aceEditor.editor.setKeyboardHandler("");
      }
    }
  }
};
</script> -->

<style>
.ace_editor.ace_autocomplete.ace_twilight {
  background-color: red;
}

.ace_editor.ace_autocomplete .ace_marker-layer .ace_active-line {
  background-color: blue;
}
.ace_twilight {
  background-color: red !important;
}
.ace_twilight .ace_completion-highlight {
  color: orange !important;
}
.ace-twilight .ace_marker-layer .ace_selection {
  background: rgb(60, 97, 146) !important;
}
</style>

<style scoped>
.container {
  padding: 12px;
}

.editor-console-wrapper {
  display: grid;
  grid-template-columns: 1fr;
  grid-template-rows: auto 3fr 170px;
  grid-row-gap: 10px;
  overflow: hidden;
}

#filename {
  text-align: center;
}

#ace-editor {
  overflow: hidden;
  justify-self: center;
}

#console {
  overflow: auto;
  font-family: Consolas, Menlo, Courier, monospace;
  margin: 15px 10px 5px 10px;
  padding: 10px;
  white-space: pre-wrap;
}

.hide-console-wrapper {
  grid-template-rows: auto 3fr 0fr;
}

.hide-console {
  display: none;
}

.text {
  font-weight: 400;
}
</style>
