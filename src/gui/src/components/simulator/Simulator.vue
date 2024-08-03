<template>
    <v-navigation-drawer permanent rail>
    <v-list-item @click="openFile()" prepend-icon="folder_open">
      <v-tooltip location="right" activator="parent" text="Open File" />
    </v-list-item>
    <v-list-item @click="toggleSimulator('run')">
      <template v-slot:prepend>
        <v-icon v-if="!sim.running" icon="play_arrow" />
        <v-icon v-else icon="pause" />
      </template>
      <v-tooltip location="right" activator="parent" v-if="!sim.running" text="Run" />
      <v-tooltip location="right" activator="parent" v-else text="Pause" />
    </v-list-item>
    <v-list-item @click="reloadFile()" prepend-icon="refresh">
      <v-tooltip location="right" activator="parent" text="Reload Object Files" />
    </v-list-item>
    <v-list-item @click="toggleSimulator('over')" prepend-icon="redo">
      <v-tooltip location="right" activator="parent" text="Step Over" />
    </v-list-item>
    <v-list-item @click="toggleSimulator('in')" prepend-icon="subdirectory_arrow_right">
      <v-tooltip location="right" activator="parent" text="Step In" />
    </v-list-item>
    <v-list-item @click="toggleSimulator('out')" prepend-icon="subdirectory_arrow_left">
      <v-tooltip location="right" activator="parent" text="Step Out" />
    </v-list-item>
    <v-list-item @click="reinitializeMachine()" prepend-icon="power_settings_new">
      <v-tooltip location="right" activator="parent" text="Reinitialize Machine" />
    </v-list-item>
    <v-list-item @click="randomizeMachine()" prepend-icon="shuffle">
      <v-tooltip location="right" activator="parent" text="Randomize Machine" />
    </v-list-item>
  </v-navigation-drawer>
  <!-- Main editor content -->
  <v-main>
    <v-container fluid class="fill-height">
      <v-snackbar v-model="isSnackBarVisible" :timeout="2500" location="top">
        <span>Object File Loaded!</span>
        <template v-slot:actions>
          <v-btn
            color="red"
            flat
            variant="text"
            @click="isSnackBarVisible = false"
          >
            Close
          </v-btn>
        </template>
      </v-snackbar>
      <v-row class="align-self-stretch">
        <v-col :cols="4" class="d-flex flex-column ga-3">
          <div>
            <h3 class="view-header">Registers</h3>
            <v-data-table
              class="elevation-4 sim-data-table"
              density="compact"
              hide-default-footer
              :items-per-page="-1"
              :items="sim.regs"
            >
              <template v-slot:colgroup>
                <colgroup>
                  <col style="width: 20%" />
                  <col style="width: 20%" />
                  <col style="width: 20%" />
                  <col style="width: 40%" />
                </colgroup>
              </template>
              <template v-slot:headers>
                <tr>
                  <th class="data-cell-text"><strong>Registers</strong></th>
                  <th class="data-cell-num"><strong>Hex</strong></th>
                  <th class="data-cell-num"><strong>Decimal</strong></th>
                  <th class="data-cell-text"><strong>ASCII / Misc</strong></th>
                </tr>
              </template>
              <template v-slot:item="{ item }">
                <tr
                  v-bind:class="{
                    'row-update-flash': item.flash,
                    'row-updated': item.updated,
                    'row-disabled': sim.running
                  }"
                >
                  <td class="data-cell-text">
                    <strong>{{ item.name.toUpperCase() }}</strong>
                  </td>
                  <td class="data-cell-num editable" @click="editValue = ($event.target as HTMLElement).textContent">
                    <span>{{
                      toHex(item.value)
                    }}</span>
                    <v-menu activator="parent" :close-on-content-click="false" :width="200">
                      <v-card>
                        <v-container>
                          <v-text-field 
                            label="Hex Value"
                            variant="underlined"
                            @focus="$event.target.select()"
                            v-model.lazy="editValue"
                            @change="
                              setDataValue($event, item, 'reg', [
                                rules.hex,
                                rules.size16bit
                              ])
                            "
                            :rules="[rules.hex, rules.size16bit]"
                          >
                          </v-text-field>
                        </v-container>
                      </v-card>
                    </v-menu>
                  </td>
                  <td class="data-cell-num editable" @click="editValue = ($event.target as HTMLElement).textContent">
                    <span>{{
                      toFormattedDec(item.value)
                    }}</span>
                    <v-menu activator="parent" :close-on-content-click="false" :width="200">
                      <v-card>
                        <v-container>
                          <v-text-field 
                            label="Decimal Value"
                            variant="underlined"
                            @focus="$event.target.select()"
                            v-model.lazy="editValue"
                            @change="
                              setDataValue($event, item, 'reg', [
                                rules.dec,
                                rules.size16bit
                              ])
                            "
                            :rules="[rules.dec, rules.size16bit]"
                          >
                          </v-text-field>
                        </v-container>
                      </v-card>
                    </v-menu>
                  </td>
                  <td class="data-cell-text">
                    <span>{{ regLabel(item) }}</span>
                  </td>
                </tr>
              </template>
            </v-data-table>
          </div>
          <div id="console-wrapper">
            <div id="console-header">
              <div id="console-title">
                <h3 class="view-header">Console (click to focus)</h3>
              </div>
              <div id="console-clear">
                <v-btn icon flat variant="text">
                  <v-icon icon="delete_forever"></v-icon>
                  <v-tooltip location="left" activator="parent" text="Clear Console" />
                </v-btn>
              </div>
            </div>
            <div
              ref="consoleRef"
              class="elevation-4 console"
              v-html="consoleStr"
              @keydown="handleConsoleInput"
              tabindex="0"
            ></div>
          </div>
        </v-col>
        <v-col :cols="8" class="d-flex flex-column justify-space-between">
          <div ref="memViewWrapper">
            <h3 class="view-header">Memory</h3>
            <v-data-table
              class="elevation-4 sim-data-table"
              hide-default-footer
              density="compact"
              :items-per-page="-1"
              :items="memView.data"
            >
              <template v-slot:colgroup>
                <colgroup>
                  <col style="width: 2em" />
                  <col style="width: 2em" />
                  <col style="width: 10%" />
                  <col style="width: 10%" />
                  <col style="width: 10%" />
                  <col style="width: 15%" />
                  <col style="width: 45%" />
                </colgroup>
              </template>
              <template v-slot:headers>
                <tr>
                  <th class="data-cell-btn"><strong>BP</strong></th>
                  <th class="data-cell-btn"><strong>PC</strong></th>
                  <th class="data-cell-num"><strong>Address</strong></th>
                  <th class="data-cell-num"><strong>Hex</strong></th>
                  <th class="data-cell-num"><strong>Decimal</strong></th>
                  <th class="data-cell-text"><strong>Label</strong></th>
                  <th class="data-cell-text"><strong>Instructions</strong></th>
                </tr>
              </template>
              <template v-slot:item="{ item }">
                <tr
                  v-bind:class="{
                    'row-update-flash': item.flash,
                    'row-updated': item.updated,
                    'row-disabled': sim.running,
                    'row-curr-pc': isPCAt(item.addr)
                  }"
                >
                  <td class="data-cell-btn">
                    <v-btn icon flat block :ripple="false" @click="toggleBreakpoint(item.addr)">
                      <v-icon
                        icon="report"
                        class="breakpoint-icon"
                        :color="isBreakpointAt(item.addr) ? 'red' : 'grey'"
                        :size="isBreakpointAt(item.addr) ? 'default' : 'small'"
                      />
                    </v-btn>
                  </td>
                  <td class="data-cell-btn">
                    <v-btn icon flat block :ripple="false" @click="setPC(item.addr)">
                      <v-icon
                        icon="play_arrow"
                        class="pc-icon"
                        :color="isPCAt(item.addr) ? 'blue' : 'grey'"
                        :size="isPCAt(item.addr) ? 'default' : 'small'"
                      />
                    </v-btn>
                  </td>
                  <td class="data-cell-num">
                    <strong>{{ toHex(item.addr) }}</strong>
                  </td>
                  <td class="data-cell-num editable" @click="editValue = ($event.target as HTMLElement).textContent">
                    <span>{{
                      toHex(item.value)
                    }}</span>
                    <v-menu activator="parent" :close-on-content-click="false" :width="200">
                      <v-card>
                        <v-container>
                          <v-text-field 
                            label="Hex Value"
                            variant="underlined"
                            @focus="$event.target.select()"
                            v-model.lazy="editValue"
                            @change="
                              setDataValue($event, item, 'mem', [
                                rules.hex,
                                rules.size16bit
                              ])
                            "
                            :rules="[rules.hex, rules.size16bit]"
                          >
                          </v-text-field>
                        </v-container>
                      </v-card>
                    </v-menu>
                  </td>
                  <td class="data-cell-num editable" @click="editValue = ($event.target as HTMLElement).textContent">
                    <span>{{
                      toFormattedDec(item.value)
                    }}</span>
                    <v-menu activator="parent" :close-on-content-click="false" :width="200">
                      <v-card>
                        <v-container>
                          <v-text-field 
                            label="Decimal Value"
                            variant="underlined"
                            @focus="$event.target.select()"
                            v-model.lazy="editValue"
                            @change="
                              setDataValue($event, item, 'mem', [
                                rules.dec,
                                rules.size16bit
                              ])
                            "
                            :rules="[rules.dec, rules.size16bit]"
                          >
                          </v-text-field>
                        </v-container>
                      </v-card>
                    </v-menu>
                  </td>
                  <td class="data-cell-text" @click="jumpToSource(item.label)">
                    <i>{{ item.label }}</i>
                  </td>
                  <td class="data-cell-text" @click="jumpToSource(item.addr)">
                    <i>{{ item.line }}</i>
                  </td>
                </tr>
              </template>
            </v-data-table>
          </div>

          <div id="controls">
            <div id="jump-to-location">
              <v-text-field
                single-line
                variant="underlined"
                label="Jump To Location"
                @change="jumpToMemViewStr()"
                v-model="jumpToLocInput"
              >
              </v-text-field>
            </div>
            <div id="jump-buttons">
              <v-btn
                @click="jumpToPC(true)"
                variant="text"
              >
                <span class="title">PC</span>
                <v-tooltip location="top" activator="parent" text="Jump to PC" />
              </v-btn>
              
              <v-btn 
                flat
                icon
                @click="jumpToPrevMemView()"
              >
                <v-icon size="x-large" icon="arrow_back" />
                <v-tooltip location="top" activator="parent" :text="toHex(toUint16(memView.start - memView.data.length))" />
              </v-btn>
              <v-btn 
                flat
                icon
                @click="jumpToPartMemView(-5)"
              >
                <v-icon icon="arrow_back" />
                <v-tooltip location="top" activator="parent" :text="toHex(toUint16(memView.start - 5))" />
              </v-btn>
              <v-btn 
                flat
                icon
                @click="jumpToPartMemView(+5)"
              >
                <v-icon icon="arrow_forward" />
                <v-tooltip location="top" activator="parent" :text="toHex(toUint16(memView.start + 5))" />
              </v-btn>
              <v-btn 
                flat
                icon
                @click="jumpToNextMemView()"
              >
                <v-icon size="x-large" icon="arrow_forward" />
                <v-tooltip location="top" activator="parent" :text="toHex(toUint16(memView.start + memView.data.length))" />
              </v-btn>
            </div>
          </div>
        </v-col>
      </v-row>
    </v-container>
  </v-main>
</template>
  
<script setup lang="ts">
import API from 'src/api';
import { useActiveFileStore } from '../../store/active_file';
import { useSettingsStore } from '../../store/settings';
import { onActivated, onMounted, onUnmounted, ref, watch } from 'vue';
import Convert from 'ansi-to-html';
import { useRouter } from 'vue-router';
declare const api: API;
const { lc3, dialog, fs } = api;

const settings = useSettingsStore();
const activeFileStore = useActiveFileStore();
const router = useRouter();

const sim = ref({
  regs: [
    { flash: false, updated: false, name: "r0", value: 0 },
    { flash: false, updated: false, name: "r1", value: 0 },
    { flash: false, updated: false, name: "r2", value: 0 },
    { flash: false, updated: false, name: "r3", value: 0 },
    { flash: false, updated: false, name: "r4", value: 0 },
    { flash: false, updated: false, name: "r5", value: 0 },
    { flash: false, updated: false, name: "r6", value: 0 },
    { flash: false, updated: false, name: "r7", value: 0 },
    { flash: false, updated: false, name: "psr", value: 0 },
    { flash: false, updated: false, name: "pc", value: 0 },
    { flash: false, updated: false, name: "mcr", value: 0 }
  ],
  breakpoints: [] as number[],
  running: false
})
const memView = ref({
  start: 0x3000,
  data: [{
    addr: 0, value: 0, line: "", label: "", flash: false, updated: false
  }],
  symTable: {} as Record<number, string>
})

const isSnackBarVisible = ref(false);
const consoleStr = ref("");
const jumpToLocInput = ref("");
let lastLoadedFile: string | null = null;
let pollOutputHandle: number | null = null;
let memScrollOffset = 0;

type RegDataRow = typeof sim.value.regs[number];
type MemDataRow = typeof memView.value.data[number];

const rules: Record<string, ValidationRule> = {
  hex(value: string) {
    return /^0?x[0-9A-Fa-f]+$/.test(value) || "Invalid hex number";
  },
  dec(value: string) {
    return /^-?\d+$/.test(value) || "Invalid decimal number";
  },
  size16bit(value: string) {
    let intValue = parseInputString(value);
    return (
      intValue === toInt16(intValue) || intValue === toUint16(intValue) ||
      "Value must be between x0000 and xFFFF"
    );
  }
}
type ValidationRule = (value: string) => boolean | string;
const editValue = ref("");

const memViewWrapper = ref(null);
watch(memViewWrapper, el => {
  el.addEventListener("wheel", handleMemoryScroll);
}, { once: true });
const consoleRef = ref<HTMLDivElement>(null);

onMounted(() => {
  refreshMemoryPanel();
  window.addEventListener("resize", refreshMemoryPanel);
})
onUnmounted(() => {
  memViewWrapper.value?.removeEventListener("wheel", handleMemoryScroll);
  window.removeEventListener("resize", refreshMemoryPanel);

})
onActivated(() => {
  let asmFileName = activeFileStore.path;
  if (asmFileName != null && activeFileStore.lastBuilt > activeFileStore.lastLoaded) {
    let objFileName = asmFileName.replace(/\.asm$/, ".obj");
    if (fs.exists(objFileName)) {
      loadFile(objFileName);
    }
    activeFileStore.touchLoadTime();
  }
})

function refreshMemoryPanel() {
  memView.value.data = Array.from(
    { length: Math.floor((window.innerHeight - 140) / 25) - 4},
    () => ({
      addr: 0,
      value: 0,
      line: "",
      label: "",
      flash: false,
      updated: false
    })
  );

  updateUI();
  jumpToPC(true);
}
function handleMemoryScroll(e: WheelEvent) {
  e.preventDefault();

  if (!lc3.isSimRunning()) {
    memScrollOffset += e.deltaY;
    if (Math.abs(memScrollOffset) > 20) {
      jumpToPartMemView(Math.floor(memScrollOffset / 20));
      memScrollOffset = 0;
    }
  }
}
async function openFile(path: string | undefined = undefined) {
  let selectedFiles: string[] = [];
  if (!path) {
    let result = await dialog.showModal("open", {
      properties: ["openFile"],
      filters: [{ name: "Object Files", extensions: ["obj"] }]
    });

    if (!result.canceled) {
      selectedFiles = result.filePaths;
    }
  } else {
    // Path already defined
    selectedFiles = [path];
  }

  if (selectedFiles.length > 0) {
    loadFile(selectedFiles[0]);
  }
}
function loadFile(path: string) {
  // clear output on file (re)load
  if (settings.clear_out_on_reload) {
    clearConsole();
  }

  lastLoadedFile = path;
  
  // load object file can fail if the object file is malformed
  let success = true;
  try {
    lc3.loadObjectFile(path);
  } catch (e) {
    success = false;
  }

  memView.value.start = lc3.getRegValue("pc");
  memView.value.symTable = lc3.getCurrSymTable();
  updateUI();
  isSnackBarVisible.value = success;
}
function reloadFile() {
  loadFile(lastLoadedFile);
  updateUI();
}
function toggleSimulator(runKind: "in" | "out" | "over" | "run") {
  if (typeof pollOutputHandle !== "number") {
    pollOutputHandle = setInterval(updateConsole, 50) as unknown as number;
  }
  
  if (!sim.value.running) {
    lc3.clearInput();
    sim.value.running = true;
    return new Promise<void>((resolve, reject) => {
      let callback = (error: Error) => {
        if (error) {
          reject(error);
          return;
        }

        endSimulation(runKind !== "run" || lc3.didHitBreakpoint());
        resolve();
      };

      if (runKind === "in") {
        lc3.stepIn(callback);
      } else if (runKind === "out") {
        lc3.stepOut(callback);
      } else if (runKind === "over") {
        lc3.stepOver(callback);
      } else if (runKind === "run") {
        lc3.run(callback);
      } else {
        let _exhaustiveCheck: never = runKind;
      }
    });
  } else {
    lc3.pause();
    endSimulation(false);
  }
}
function reinitializeMachine() {
  lc3.reinitializeMachine();
  clearConsole();
  updateUI();
}
function randomizeMachine() {
  lc3.randomizeMachine();
  clearConsole();
  updateUI();
}
function endSimulation(jumpToPC_: boolean) {
  clearInterval(pollOutputHandle);
  pollOutputHandle = null;

  lc3.clearInput();
  sim.value.running = false;
  updateUI(true);
  sim.value.regs[9].value = lc3.getRegValue("pc");

  if (jumpToPC_) jumpToPC(false);
}
function clearConsole() {
  consoleStr.value = "";
  lc3.clearOutput();
}
function handleConsoleInput(e: KeyboardEvent) {
  // Typable characters on a standard keyboard.
  const overrides: Record<string, string> = {
    Enter: '\n',
    Backspace: '\x08',
    Tab: '\x09',
    Escape: '\x1b',
    Delete: '\x7f'
  };

  // TODO: since the console string is rendered as I/O, 
  // the console actually allows for "HTML injection"
  let key = e.key;
  if (key in overrides) {
    lc3.addInput(overrides[key]);
  } else if (key.length === 1) {
    // Handle CTRL-a through CTRL-z.
    let code = key.charCodeAt(0);
    if (code > 64 && code < 128 && e.ctrlKey) {
      key = String.fromCharCode(code & 0x1F);
    } 
    lc3.addInput(key);
  }

  e.preventDefault(); // for TAB, etc.
}

function setDataValue(event: Event, dataCell: RegDataRow, type: "reg", rules: ValidationRule[]): void;
function setDataValue(event: Event, dataCell: MemDataRow, type: "mem", rules: ValidationRule[]): void;
function setDataValue(event: Event, dataCell: RegDataRow | MemDataRow, type: "reg" | "mem", rules: ValidationRule[]) {
  let value = (event.target as HTMLInputElement).value;
  let validated = rules.every(r => r(value) === true);
  
  // Validation failed, so ignore set
  if (!validated) {
    if (type === "reg" && "name" in dataCell) {
      dataCell.value = lc3.getRegValue(dataCell.name);
    } else if (type === "mem" && "addr" in dataCell) {
      dataCell.value = lc3.getMemValue(dataCell.addr);
    }
    return;
  }

  dataCell.value = parseInputString(value);
  if (type === "reg" && "name" in dataCell) {
    lc3.setRegValue(dataCell.name, dataCell.value);
  } else if (type === "mem" && "addr" in dataCell) {
    lc3.setMemValue(dataCell.addr, dataCell.value);
  }
  
  updateUI();
}
function updateUI(showUpdates = false, updateReg = true) {
  // Can't update UI while it's running
  if (lc3.isSimRunning()) return;

  // Registers
  if (updateReg) {
    for (let reg of sim.value.regs) {
      const regVal = lc3.getRegValue(reg.name);
      const prevVal = reg.value;

      reg.value = regVal;
      // flash and highlight registers that change from their previous values
      reg.flash = false;
      reg.updated = false;
      if (showUpdates) {
        const updated = reg.name === "pc" ? regVal !== prevVal + 1 : regVal !== prevVal;
        if (updated) {
          reg.flash = true;
          setTimeout(() => {
            reg.flash = false;
            reg.updated = true;
          }, 250);
        }
      }
    }
  }

  // Memory
  for (let i = 0; i < memView.value.data.length; i++) {
    let addr = toUint16(memView.value.start + i);
    const dataLine = memView.value.data[i];

    const memVal = lc3.getMemValue(addr);
    const prevVal = dataLine.value;

    dataLine.addr = addr;
    dataLine.value = memVal;
    dataLine.line = lc3.getMemLine(addr);
    // show label using symbol table
    dataLine.label = memView.value.symTable[addr]?.toUpperCase() ?? "";
  
    // hack to highlight changed values within current display
    // (lc3tools CLI doesn't track change "history" across all memory)
    dataLine.flash = false;
    dataLine.updated = false;
    if (showUpdates && memVal !== prevVal) {
      dataLine.flash = true;
      setTimeout(() => {
        dataLine.flash = false;
        dataLine.updated = true;
      }, 250);
    }
  }

  updateConsole();
}
function updateConsole() {
  // Console

  // TODO: reduce rendundancy by having these defined once
  // see [`Editor.vue#build`].

  // VS Code's Dark+ terminal colors.
  let convert = new Convert({
    colors: [
    "#000000", "#CD3131", "#0DBC79", "#E5E510", 
    "#2472C8", "#BC3FBC", "#11A8CD", "#E5E5E5", 
    "#666666", "#F14C4C", "#23D18B", "#F5F543", 
    "#3B8EEA", "#D670D6", "#29B8DB", "#E5E5E5"
    ]
  });
  let update = lc3.getAndClearOutput();
  if (update.length) {
    // Resolve all internal backspaces first
    while (update.match(/[^\x08\n]\x08/)) {
      update = update.replace(/[^\x08\n]\x08/g, "");
    }
    let bs = 0; // backspace count
    while (
      update.charAt(bs) === "\x08" &&
      bs < consoleStr.value.length &&
      consoleStr.value.slice(-(1 + bs), -bs) !== "\n"
    ) {
      bs++;
    }
    if (bs) {
      update = update.substring(bs);
      consoleStr.value = consoleStr.value.slice(0, -bs);
    }
    consoleStr.value += convert.toHtml(update);
    setTimeout(
      () => (consoleRef.value.scrollTop = consoleRef.value.scrollHeight)
    );
  }
}
function toggleBreakpoint(addr: number) {
  let idx = sim.value.breakpoints.indexOf(addr);

  if (!lc3.isSimRunning()) {
    if (idx == -1) {
      lc3.setBreakpoint(addr);
      sim.value.breakpoints.push(addr);
    } else {
      lc3.removeBreakpoint(addr);
      sim.value.breakpoints.splice(idx, 1);
    }
  }
}
function setPC(addr: number) {
  if (!lc3.isSimRunning()) {
    lc3.setRegValue("pc", toUint16(addr));
    lc3.restartMachine();
    updateUI();
  }
}
function jumpToSource(location: string | number) {
  if (!lc3.isSimRunning()) {
    let span;
    if (typeof location === "string") {
      span = lc3.getLabelSourceRange(location);
    } else if (typeof location === "number") {
      span = lc3.getAddrSourceRange(location);
    } else {
      let _exhaustiveCheck: never = location;
    }

    if (typeof span !== "undefined") {
      let [slno, scno, elno, ecno] = span;
      router.push({ name: "editor", hash: `#L${slno}C${scno}-L${elno}C${ecno}` });
    }
  }
}
function isBreakpointAt(addr: number) {
  return sim.value.breakpoints.includes(addr)
}
function isPCAt(addr: number) {
  return addr == sim.value.regs[9].value && !sim.value.running;
}

// Memory view jump functions
function jumpToMemView(newStart: number) {
  memView.value.start = toUint16(newStart);
  updateUI(false, false);
}
function jumpToMemViewStr() {
  let match = jumpToLocInput.value.match(/^(?:0?[xX])?([0-9A-Fa-f]+)$/);
  if (match != null) {
    jumpToMemView(parseInt(match[1], 16));
  }
}
function jumpToPartMemView(offset: number) {
  jumpToMemView(memView.value.start + offset);
}
function jumpToPrevMemView() {
  jumpToPartMemView(-memView.value.data.length);
}
function jumpToNextMemView() {
  jumpToPartMemView(+memView.value.data.length);
}
function jumpToPC(jumpIfInView: boolean) {
  let pc = toUint16(sim.value.regs[9].value);
  let memViewStart = memView.value.start;
  let memViewEnd = memViewStart + memView.value.data.length;
  
  let pcInView = memViewStart <= pc && pc < memViewEnd;
  if (jumpIfInView || !pcInView) jumpToMemView(pc);
}

// Helper functions
function psrToCC(psr: number) {
  let cc = psr & 0b111;
  switch (cc) {
    case 0b100: return "N"
    case 0b010: return "Z"
    case 0b001: return "P"
    default: return "?"
  }
}
function toHex(value: number) {
  let hex = value.toString(16).toUpperCase();
  return `x${hex.padStart(4, "0")}`;
}
function toFormattedDec(value: number) {
  if (settings.numbers === "signed") {
    return toInt16(value);
  } else if (settings.numbers === "unsigned") {
    return toUint16(value);
  } else {
    let _exhaustiveCheck: never = settings.numbers;
  }
}
function parseInputString(value: string) {
  if (value.startsWith("x")) value = "0" + value;
  return parseInt(value);
}
function regLabel(item: RegDataRow) {
  if (item.name === "psr") {
    return "CC: " + psrToCC(item.value);
  } else if (item.name.startsWith("r") && 0 <= item.value && item.value <= 127) {
    return String.fromCharCode(item.value);
  }

  return "";
}

function toUint16(value: number) {
  return value & 0xFFFF;
}
function toInt16(value: number) {
  return (value << 16) >> 16;
}
</script>


<style scoped>
.view-header {
  text-align: center;
  padding-bottom: 5px;
}

/* Generic data table styles */
.sim-data-table {
  /* Supercompact! */
  --v-table-header-height: 29px;
  --v-table-row-height: 25px;
}
.sim-data-table:deep(table) {
  /* Propagates this property into the <table> element of the <v-data-table> component */
  table-layout: fixed;
}

.sim-data-table tr {
  transition: background-color 0.25s ease-in-out;
}
.sim-data-table thead tr {
  background-color: #00000040;
  column-gap: 5px;
}
.sim-data-table tbody tr {
  font-family: Consolas, Menlo, Courier, monospace;
  /* Force row to be 1 line wide */
  overflow: hidden;
  white-space: nowrap;
}
.sim-data-table tbody tr:hover {
  background-color: #7f7f7f4d;
}
.row-update-flash {
  background-color: #fff700a0;
}
.row-updated {
  background-color: #fff70038;
}
.row-disabled {
  background-color: lightgrey !important;
}

.data-cell-text {
  text-align: left !important;
}
.data-cell-btn {
  text-align: center !important;
}
.data-cell-btn > * {
  display: block;
  margin: auto;
}
.data-cell-btn * {
  transition: color 0.2s, font-size 0.2s;
}
.data-cell-btn:deep(button) {
  background-color: transparent;
  /* Force height of buttons to be smaller than the height of each row */
  height: calc(var(--v-table-row-height) - 1);
}
.data-cell-num {
  text-align: right !important;
}

/* Console styles */
#console-wrapper {
  display: flex;
  flex-direction: column;
  flex: 1;
}

#console-header {
  display: grid;
  grid-template-columns: 50px auto 50px;
  grid-template-rows: 100%;
  justify-items: center;
  align-items: center;
  overflow: hidden;
}

#console-title {
  grid-column: 2;
  grid-row: 1;
}

#console-clear {
  grid-column: 3;
  grid-row: 1;
}

.console {
  flex: 1;
  font-family: Consolas, Menlo, Courier, monospace;
  padding: 8px;
  overflow-y: scroll;
  white-space: pre-wrap;
  background-color: rgb(var(--v-theme-surface));
}

.console:focus {
  outline: none;
  box-shadow: 0px 0px 6px 3px rgba(33, 150, 223, 0.6) !important;
}

.console::after {
  content: "\25af";
}
.console:focus::after {
  content: "\25ae";
}

/* Memory view styles */
.row-curr-pc {
  background-color: #008cff4d;
}

.breakpoint-icon:hover {
  color: red !important;
}

.pc-icon:hover {
  color: #2196f3 !important;
}

/* Memory view controls styles */
#controls {
  flex-basis: content;
  order: 2;

  display: grid;
  grid-template-columns: 30% auto;
  grid-template-rows: auto;
  align-items: center;
}

#jump-to-location {
  grid-column: 1;
  grid-row: 1;
}

#jump-buttons {
  grid-column: 2;
  grid-row: 1;
  text-align: right;
}
</style>
