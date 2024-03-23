<!-- Copyright 2020 McGraw-Hill Education. All rights reserved. No reproduction or distribution without the prior written consent of McGraw-Hill Education. -->
<template>
  <v-app id="editor" v-bind:dark="darkMode">
    <!-- Sidebar -->
    <v-navigation-drawer fixed mini-variant permanent app>
      <v-list two-line>
        <v-tooltip right>
          <v-list-tile slot="activator" @click="openFile()">
            <v-list-tile-action>
              <v-icon large>folder_open</v-icon>
            </v-list-tile-action>
          </v-list-tile>
          <span>Open File</span>
        </v-tooltip>
        <v-tooltip right>
          <v-list-tile slot="activator" @click="saveFile()">
            <v-list-tile-action>
              <v-badge color="orange darken-2" overlap>
                <v-icon large>save</v-icon>
                <span v-if="editor.content_changed" slot="badge"
                  ><strong>!</strong></span
                >
              </v-badge>
            </v-list-tile-action>
          </v-list-tile>
          <span>Save File</span>
        </v-tooltip>
        <v-tooltip right>
          <v-list-tile slot="activator" @click="saveFileAs()">
            <v-list-tile-action>
              <v-icon large>note_add</v-icon>
            </v-list-tile-action>
          </v-list-tile>
          <span>Save File As</span>
        </v-tooltip>
        <v-tooltip right>
          <v-list-tile slot="activator" @click="build()">
            <v-list-tile-action>
              <v-icon large>build</v-icon>
            </v-list-tile-action>
          </v-list-tile>
          <span v-if="this.$store.getters.activeFilePath === null"
            >Assemble or Convert</span
          >
          <span v-else-if="this.$store.getters.activeFilePath.endsWith('.asm')"
            >Assemble</span
          >
          <span v-else-if="this.$store.getters.activeFilePath.endsWith('.bin')"
            >Convert</span
          >
          <span v-else>Build</span>
        </v-tooltip>
        <v-tooltip right>
          <v-list-tile slot="activator" @click="toggleConsole()">
            <v-list-tile-action>
              <v-icon large>terminal</v-icon>
            </v-list-tile-action>
          </v-list-tile>
          <span>Toggle Console Visibility</span>
        </v-tooltip>
      </v-list>
    </v-navigation-drawer>

    <!-- Main editor content -->
    <v-content>
      <v-container fluid fill-height>
        <v-layout row wrap>
          <v-flex
            xs12
            shrink
            class="editor-console-wrapper"
            :class="{ 'hide-console-wrapper': !show_console }"
          >
            <h3 id="filename" class="view-header">{{ getFilename }}</h3>
            <ace-editor
              id="ace-editor"
              class="elevation-2"
              v-model="editor.current_content"
              @init="editorInit"
              lang="lc3"
              v-bind:theme="darkMode ? 'twilight' : 'textmate'"
              height="100%"
              width="100%"
              ref="aceEditor"
            />
            <div
              :class="{ 'hide-console': !show_console }"
              id="console"
              class="elevation-4"
              v-html="console_str"
            ></div>
          </v-flex>
        </v-layout>
      </v-container>
    </v-content>
  </v-app>
</template>

<script>
import { remote } from "electron";
import path from "path";
import Vue from "vue";
import Vuetify from "vuetify";
import fs from "fs";
import ace from "brace";
import { CreateLc3CompletionProvider } from "./completions";

import * as lc3 from "lc3-backend";
import * as Convert from "ansi-to-html";

Vue.use(Vuetify);

export default {
  name: "editor",
  data: () => {
    return {
      editor: {
        original_content: "",
        current_content: "",
        content_changed: false
      },
      console_str: "",
      editor_theme: "textmate",
      show_console: false,
      ace_editor: null
    };
  },
  components: {
    "ace-editor": require("vue2-ace-editor-electron")
  },
  mounted() {
    setInterval(this.autosaveFile, 5 * 60 * 1000); // autosave every 5 minutes (cool!)
  },
  beforeRouteEnter(to, from, next) {
    next(vm => {
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
  
          let { Range } = ace.acequire("ace/range");
          vm.ace_editor.gotoLine(slno, scno, true);
          vm.ace_editor.getSelection().setRange(new Range(slno, scno, elno, ecno));
        }
      }
    });
  },
  methods: {
    toggleConsole() {
      this.show_console = !this.show_console;
    },
    saveFileAs() {
      // Todo: try catch around this
      let new_file = remote.dialog.showSaveDialogSync({
        filters: [
          { name: "Assembly", extensions: ["asm"] },
          { name: "Binary", extensions: ["bin"] }
        ]
      });

      // Guard against user cancelling
      if (new_file) {
        fs.writeFileSync(new_file, this.editor.current_content);
        this.openFile(new_file);
      }
    },
    saveFile() {
      // Todo: try catch around this
      // If we don't have a file, create one
      if (this.$store.getters.activeFilePath === null) {
        this.saveFileAs();
      } else {
        fs.writeFileSync(
          this.$store.getters.activeFilePath,
          this.editor.current_content
        );
        this.editor.original_content = this.editor.current_content;
      }
      this.build();
    },
    autosaveFile() {
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
    },
    openFile(path) {
      // Todo: try catch around this
      // if not given a path, open a dialog to ask user for file
      let selected_files = [];
      if (path === undefined || typeof path !== "string") {
        selected_files = remote.dialog.showOpenDialogSync({
          properties: ["openFile"],
          filters: [
            { name: "Assembly", extensions: ["asm"] },
            { name: "Binary", extensions: ["bin"] }
          ]
        });
      } else {
        selected_files = [path];
      }

      // Dialog returns an array of files, we only care about the first one
      if (selected_files) {
        let active_file = selected_files[0];
        this.editor.original_content = this.editor.current_content = fs.readFileSync(
          active_file,
          "utf-8"
        );
        this.$store.commit("setActiveFilePath", active_file);
      }
    },
    build() {
      // save the file if it hasn't been saved
      if (this.editor.content_changed) {
        this.editor.content_changed = false;
        this.saveFile();
      }
      // show console when assembling
      this.show_console = true;
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
    },
    editorInit(editor) {
      require("./lc3");
      require("brace/mode/html");
      require("brace/mode/javascript");
      require("brace/mode/less");
      require("brace/theme/textmate");
      require("brace/theme/twilight");
      require("brace/ext/searchbox");
      require("brace/keybinding/vim");
      require("brace/ext/language_tools"); // for more config: const langTools = ace.acequire("ace/ext/language_tools");
      
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
        exec: this.saveFile
      });
      editor.commands.addCommand({
        name: "build",
        bindKey: { win: "Ctrl-Enter", mac: "Cmd-Enter" },
        exec: this.build
      });
      editor.commands.addCommand({
        name: "open",
        bindKey: { win: "Ctrl-O", mac: "Cmd-O" },
        exec: this.openFile
      });

      this.ace_editor = editor;
    }
  },
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
</script>

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
