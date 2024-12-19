<template>
  <div 
    class="contents sim-top"
    :class="{
      'reduce-flashing': settings.reduce_flashing
    }"
  >
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
      <v-list-item @click="toggleSimulator('run')">
        <template #prepend>
          <v-icon
            v-if="!sim.running"
            :icon="mdiPlay"
          />
          <v-icon
            v-else
            :icon="mdiPause"
          />
        </template>
        <v-tooltip
          v-if="!sim.running"
          location="right"
          activator="parent"
          text="Run"
        />
        <v-tooltip
          v-else
          location="right"
          activator="parent"
          text="Pause"
        />
      </v-list-item>
      <v-list-item
        :prepend-icon="mdiRefresh"
        @click="reloadFile()"
      >
        <v-tooltip
          location="right"
          activator="parent"
          text="Reload Object Files"
        />
      </v-list-item>
      <v-divider />
      <v-list-item
        :prepend-icon="mdiDebugStepOver"
        @click="toggleSimulator('over')"
      >
        <v-tooltip
          location="right"
          activator="parent"
          text="Step Over"
        />
      </v-list-item>
      <v-list-item
        :prepend-icon="mdiDebugStepInto"
        @click="toggleSimulator('in')"
      >
        <v-tooltip
          location="right"
          activator="parent"
          text="Step In"
        />
      </v-list-item>
      <v-list-item
        :prepend-icon="mdiDebugStepOut"
        @click="toggleSimulator('out')"
      >
        <v-tooltip
          location="right"
          activator="parent"
          text="Step Out"
        />
      </v-list-item>
      <v-divider />
      <v-list-item
        :prepend-icon="mdiPower"
        @click="reinitializeMachine()"
      >
        <v-tooltip
          location="right"
          activator="parent"
          text="Reinitialize Machine"
        />
      </v-list-item>
      <v-list-item
        :prepend-icon="mdiShuffle"
        @click="randomizeMachine()"
      >
        <v-tooltip
          location="right"
          activator="parent"
          text="Randomize Machine"
        />
      </v-list-item>
    </v-navigation-drawer>
    <!-- Main editor content -->
    <v-main
      @drop.prevent="dropFile"
      @dragover.prevent
    >
      <v-container
        fluid
        class="fill-height"
      >
        <v-snackbar
          v-model="isSnackBarVisible"
          :timeout="2500"
          location="top"
        >
          <span>Object File Loaded!</span>
          <template #actions>
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
          <v-col
            :cols="4"
            class="d-flex flex-column ga-3 h-limit"
          >
            <div>
              <div class="header-bar">
                <h3 class="header-bar-title">
                  Registers
                </h3>
              </div>
              <v-data-table
                class="elevation-4 sim-data-table"
                density="compact"
                hide-default-footer
                :items-per-page="-1"
                :items="sim.regs"
              >
                <template #colgroup>
                  <colgroup>
                    <col style="width: 20%">
                    <col style="width: 20%">
                    <col style="width: 20%">
                    <col style="width: 40%">
                  </colgroup>
                </template>
                <template #headers>
                  <tr>
                    <th class="data-cell-text">
                      <strong>Registers</strong>
                    </th>
                    <th class="data-cell-num">
                      <strong>Hex</strong>
                    </th>
                    <th class="data-cell-num">
                      <strong>Decimal</strong>
                    </th>
                    <th class="data-cell-text">
                      <strong>ASCII / Misc</strong>
                    </th>
                  </tr>
                </template>
                <template #item="{ item }">
                  <tr
                    :class="{
                      'row-update-flash': item.flash,
                      'row-updated': item.updated,
                      'row-disabled': sim.running
                    }"
                    @contextmenu="openRegContextMenu(item)"
                  >
                    <td
                      class="data-cell-text"
                    >
                      <strong>{{ item.name.toUpperCase() }}</strong>
                    </td>
                    <td
                      class="data-cell-num clickable"
                      @click="editValue = ($event.target as HTMLElement).textContent"
                    >
                      <span>{{
                        toHex(item.value)
                      }}</span>
                      <v-menu 
                        v-if="!sim.running"
                        activator="parent" 
                        :close-on-content-click="false" 
                        :width="200"
                      >
                        <v-card>
                          <v-container>
                            <v-text-field 
                              v-model.lazy="editValue"
                              label="Hex Value"
                              variant="underlined"
                              :rules="[rules.hex, rules.size16bit]"
                              @focus="$event.target.select()"
                              @change="
                                setDataValue(item, 'reg', [
                                  rules.hex,
                                  rules.size16bit
                                ])
                              "
                            />
                          </v-container>
                        </v-card>
                      </v-menu>
                    </td>
                    <td
                      class="data-cell-num clickable"
                      @click="editValue = ($event.target as HTMLElement).textContent"
                    >
                      <span>{{
                        toFormattedDec(item.value)
                      }}</span>
                      <v-menu
                        v-if="!sim.running"
                        activator="parent" 
                        :close-on-content-click="false" 
                        :width="200"
                      >
                        <v-card>
                          <v-container>
                            <v-text-field 
                              v-model.lazy="editValue"
                              label="Decimal Value"
                              variant="underlined"
                              :rules="[rules.dec, rules.size16bit]"
                              @focus="$event.target.select()"
                              @change="
                                setDataValue(item, 'reg', [
                                  rules.dec,
                                  rules.size16bit
                                ])
                              "
                            />
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
              <div class="header-bar">
                <h3 class="header-bar-title">
                  Console (click to focus)
                </h3>
                <v-btn
                  class="header-bar-right"
                  icon
                  flat
                  variant="text"
                  @click="clearConsoleOutput()"
                >
                  <v-icon :icon="mdiDelete" />
                  <v-tooltip
                    location="left"
                    activator="parent"
                    text="Clear Console"
                  />
                </v-btn>
              </div>
              <console 
                v-model="consoleStr"
                float="bottom"
                show-focus
                show-cursor
                @keydown="handleConsoleInput"
              />
            </div>
          </v-col>
          <v-col
            :cols="8"
            class="d-flex flex-column justify-space-between"
          >
            <div ref="memViewWrapper">
              <div class="header-bar">
                <h3 class="header-bar-title">
                  Memory
                </h3>
                <v-btn
                  class="header-bar-right"
                  icon
                  flat
                  :variant="timerBtnVariant"
                  :color="timerBtnColor"
                  @click="resetTimerInputs()"
                >
                  <v-badge
                    v-model="timerRemBadgeShow"
                    location="top start"
                  >
                    <template #badge>
                      <strong>{{ sim.timer.remaining || '' }}</strong>
                    </template>
                    <v-icon :icon="mdiTimer" />
                  </v-badge>
                  <v-tooltip
                    location="left"
                    activator="parent"
                    text="Configure Timer Interrupt"
                  />
                  <v-menu
                    activator="parent"
                    :close-on-content-click="false"
                  >
                    <v-card>
                      <v-container>
                        <!-- Should use v-row, v-col, but those are grid not flex -->
                        <div class="d-flex justify-space-between align-center ga-5">
                          <h3 class="flex-grow-1">
                            Enable timer interrupt
                          </h3>
                          <v-switch
                            v-model="sim.timer.enabled"
                            class="flex-shrink-1"
                            color="primary"
                            hide-details
                            @change="setTimerStatus()"
                          />
                        </div>
                        <div class="d-flex justify-space-between align-center ga-5">
                          <h3 class="flex-grow-1">
                            Hide timer badge
                          </h3>
                          <v-switch
                            v-model="sim.timer.hide_badge"
                            class="flex-shrink-1"
                            color="primary"
                            hide-details
                            :disabled="!sim.timer.enabled"
                          />
                        </div>
                        <v-container>
                          <v-row>
                            <v-col class="align-self-center">
                              <h3>
                                Vector
                              </h3>
                            </v-col>
                            <v-col class="py-0">
                              <v-form @submit.prevent="setTimerProperty($event, 'vect')">
                                <v-text-field
                                  v-model="timerInputs.vect"
                                  color="primary"
                                  variant="underlined"
                                  :disabled="!sim.timer.enabled"
                                  :rules="[rules.hex, rules.size8bit]"
                                />
                              </v-form>
                            </v-col>
                          </v-row>
                          <v-row>
                            <v-col class="align-self-center">
                              <h3>
                                Priority
                              </h3>
                            </v-col>
                            <v-col class="py-0">
                              <v-form @submit.prevent="setTimerProperty($event, 'priority')">
                                <v-text-field
                                  v-model="timerInputs.priority"
                                  color="primary"
                                  variant="underlined"
                                  :disabled="!sim.timer.enabled"
                                  :rules="[rules.dec, rangeRule(0, 7)]"
                                />
                              </v-form>
                            </v-col>
                          </v-row>
                          <v-row>
                            <v-col class="align-self-center">
                              <h3>
                                Repeat
                              </h3>
                            </v-col>
                            <v-col class="py-0">
                              <v-form @submit.prevent="setTimerProperty($event, 'max')">
                                <v-text-field
                                  v-model="timerInputs.max"
                                  color="primary"
                                  variant="underlined"
                                  :disabled="!sim.timer.enabled"
                                  :rules="[rules.dec, rangeRule(0, 2**31 - 1)]"
                                />
                              </v-form>
                            </v-col>
                          </v-row>
                        </v-container>
                        <div class="d-flex justify-space-between align-center">
                          <h3>
                            Interrupt activates in {{ sim.timer.enabled ? sim.timer.remaining : "-" }} instruction{{ sim.timer.remaining !== 1 ? 's' : '' }}
                          </h3>
                        </div>
                        <div class="d-flex justify-end pt-3">
                          <v-btn
                            variant="flat"
                            color="primary"
                            :disabled="!sim.timer.enabled"
                            @click="resetTimer"
                          >
                            Reset
                          </v-btn>
                        </div>
                      </v-container>
                    </v-card>
                  </v-menu>
                </v-btn>
              </div>
              <v-data-table
                class="elevation-4 sim-data-table"
                hide-default-footer
                density="compact"
                :items-per-page="-1"
                :items="memView.data"
              >
                <template #colgroup>
                  <colgroup>
                    <col style="width: 2em">
                    <col style="width: 2em">
                    <col style="width: 10%">
                    <col style="width: 10%">
                    <col style="width: 10%">
                    <col style="width: 15%">
                    <col style="width: 45%">
                  </colgroup>
                </template>
                <template #headers>
                  <tr>
                    <th class="data-cell-btn">
                      <strong>BP</strong>
                    </th>
                    <th class="data-cell-btn">
                      <strong>PC</strong>
                    </th>
                    <th class="data-cell-num">
                      <strong>Address</strong>
                    </th>
                    <th class="data-cell-num">
                      <strong>Hex</strong>
                    </th>
                    <th class="data-cell-num">
                      <strong>Decimal</strong>
                    </th>
                    <th class="data-cell-text">
                      <strong>Label</strong>
                    </th>
                    <th class="data-cell-text">
                      <strong>Instructions</strong>
                    </th>
                  </tr>
                </template>
                <template #item="{ item }">
                  <tr
                    :class="{
                      'row-update-flash': item.flash,
                      'row-updated': item.updated,
                      'row-disabled': sim.running,
                      'row-curr-pc': isPCAt(item.addr)
                    }"
                    @contextmenu="openMemContextMenu(item)"
                  >
                    <td class="data-cell-btn">
                      <v-btn
                        icon
                        flat
                        block
                        :ripple="false"
                        @click="toggleBreakpoint(item.addr)"
                      >
                        <v-icon
                          :icon="mdiAlertOctagon"
                          class="breakpoint-icon"
                          :color="isBreakpointAt(item.addr) ? 'red' : 'grey'"
                          :size="isBreakpointAt(item.addr) ? 'default' : 'small'"
                        />
                      </v-btn>
                    </td>
                    <td class="data-cell-btn">
                      <v-btn
                        icon
                        flat
                        block
                        :ripple="false"
                        @click="setPC(item.addr)"
                      >
                        <v-icon
                          :icon="mdiPlay"
                          class="pc-icon"
                          :color="isPCAt(item.addr) ? 'blue' : 'grey'"
                          :size="isPCAt(item.addr) ? 'default' : 'small'"
                        />
                      </v-btn>
                    </td>
                    <td class="data-cell-num">
                      <strong>{{ toHex(item.addr) }}</strong>
                    </td>
                    <td
                      class="data-cell-num clickable"
                      @click="editValue = ($event.target as HTMLElement).textContent"
                    >
                      <span>{{
                        toHex(item.value)
                      }}</span>
                      <v-menu
                        v-if="!sim.running"
                        activator="parent" 
                        :close-on-content-click="false" 
                        :width="200"
                      >
                        <v-card>
                          <v-container>
                            <v-text-field 
                              v-model.lazy="editValue"
                              label="Hex Value"
                              variant="underlined"
                              :rules="[rules.hex, rules.size16bit]"
                              @focus="$event.target.select()"
                              @change="
                                setDataValue(item, 'mem', [
                                  rules.hex,
                                  rules.size16bit
                                ])
                              "
                            />
                          </v-container>
                        </v-card>
                      </v-menu>
                    </td>
                    <td
                      class="data-cell-num clickable"
                      @click="editValue = ($event.target as HTMLElement).textContent"
                    >
                      <span>{{
                        toFormattedDec(item.value)
                      }}</span>
                      <v-menu
                        v-if="!sim.running"
                        activator="parent" 
                        :close-on-content-click="false" 
                        :width="200"
                      >
                        <v-card>
                          <v-container>
                            <v-text-field 
                              v-model.lazy="editValue"
                              label="Decimal Value"
                              variant="underlined"
                              :rules="[rules.dec, rules.size16bit]"
                              @focus="$event.target.select()"
                              @change="
                                setDataValue(item, 'mem', [
                                  rules.dec,
                                  rules.size16bit
                                ])
                              "
                            />
                          </v-container>
                        </v-card>
                      </v-menu>
                    </td>
                    <td
                      class="data-cell-text"
                      :class="{
                        'clickable': item.label.trim().length != 0
                      }"
                      @click="jumpToSource(item.label)"
                    >
                      <i>{{ item.label }}</i>
                    </td>
                    <td 
                      class="data-cell-text" 
                      :class="{
                        'clickable': item.line.trim().length != 0
                      }"
                      @click="jumpToSource(item.addr)"
                    >
                      <i>{{ item.line }}</i>
                    </td>
                  </tr>
                </template>
              </v-data-table>
            </div>
  
            <div id="controls">
              <div id="jump-to-location">
                <v-form @submit.prevent="jumpToMemViewStr()">
                  <v-text-field
                    v-model="jumpToLocInput"
                    single-line
                    variant="underlined"
                    label="Jump To Location"
                  />
                </v-form>
              </div>
              <div id="jump-buttons">
                <v-btn
                  variant="flat"
                  class="mr-3"
                  @click="jumpToPC(true)"
                >
                  <span class="title">PC</span>
                  <v-tooltip
                    location="top"
                    activator="parent"
                    text="Jump to PC"
                  />
                </v-btn>
                
                <v-btn 
                  flat
                  icon
                  @click="jumpToPrevMemView()"
                >
                  <v-icon
                    size="x-large"
                    :icon="mdiArrowLeft"
                  />
                  <v-tooltip
                    location="top"
                    activator="parent"
                    :text="toHex(toUint16(memView.start - memView.data.length))"
                  />
                </v-btn>
                <v-btn 
                  flat
                  icon
                  @click="jumpToPartMemView(-5)"
                >
                  <v-icon :icon="mdiArrowLeft" />
                  <v-tooltip
                    location="top"
                    activator="parent"
                    :text="toHex(toUint16(memView.start - 5))"
                  />
                </v-btn>
                <v-btn 
                  flat
                  icon
                  @click="jumpToPartMemView(+5)"
                >
                  <v-icon :icon="mdiArrowRight" />
                  <v-tooltip
                    location="top"
                    activator="parent"
                    :text="toHex(toUint16(memView.start + 5))"
                  />
                </v-btn>
                <v-btn 
                  flat
                  icon
                  @click="jumpToNextMemView()"
                >
                  <v-icon
                    size="x-large"
                    :icon="mdiArrowRight"
                  />
                  <v-tooltip
                    location="top"
                    activator="parent"
                    :text="toHex(toUint16(memView.start + memView.data.length))"
                  />
                </v-btn>
              </div>
            </div>
          </v-col>
        </v-row>
      </v-container>
    </v-main>
  </div>
</template>
  
<script setup lang="ts">
import { useActiveFileStore } from '../../store/active_file';
import { useSettingsStore } from '../../store/settings';
// Vue stuff
import { computed, onActivated, onMounted, onUnmounted, ref, useTemplateRef, watch } from 'vue';
import { useRouter } from 'vue-router';
import "vuetify/components";
//
import Console from '../Console.vue';
import { mdiAlertOctagon, mdiArrowLeft, mdiArrowRight, mdiDebugStepInto, mdiDebugStepOut, mdiDebugStepOver, mdiDelete, mdiFolderOpen, mdiPause, mdiPlay, mdiPower, mdiRefresh, mdiShuffle, mdiTimer } from '@mdi/js';

const { lc3, dialog, fs } = window.api;

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
  running: false,
  timer: {
    enabled: false,
    hide_badge: false,
    vect: 0x81,
    priority: 4,
    remaining: 0,
    max: 0,
  }
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
const timerInputs = ref({
  vect: "x81",
  priority: "4",
  max: "50"
});

const timerRemBadgeShow = computed(() => sim.value.timer.enabled && !sim.value.timer.hide_badge && (sim.value.running || sim.value.timer.remaining != 0));
const timerBtnVariant = computed(() => {
  if (sim.value.timer.enabled) {
    if (!sim.value.running && sim.value.timer.remaining == 0) return "flat";
    return "tonal";
  }
  return "text";
});
const timerBtnColor = computed(() => sim.value.timer.enabled ? "primary" : undefined);
let lastLoadedFile: string | null = null;
let pollOutputHandle: ReturnType<typeof setInterval> | null = null;
let memScrollOffset = 0;

type RegDataRow = typeof sim.value.regs[number];
type MemDataRow = typeof memView.value.data[number];

const rangeRule = (min: number, max: number) => (value: string) => {
  const intValue = parseInputString(value);
  return min <= intValue && intValue <= max || `Value must be between ${min} and ${max}`;
};
const rules: Record<string, ValidationRule> = {
  hex(value: string) {
    return /^0?[xX][0-9A-Fa-f]+$/.test(value) || "Invalid hex number";
  },
  dec(value: string) {
    return /^-?\d+$/.test(value) || "Invalid decimal number";
  },
  size16bit(value: string) {
    const intValue = parseInputString(value);
    return (
      intValue === toInt16(intValue) || intValue === toUint16(intValue) ||
      "Value must be between x0000 and xFFFF"
    );
  },
  size8bit(value: string) {
    const intValue = parseInputString(value);
    return intValue === (intValue & 0xFF) || "Value must be between x00 and xFF";
  }
}
type ValidationRule = (value: string) => boolean | string;
const editValue = ref("");

const memViewWrapper = useTemplateRef("memViewWrapper");
watch(memViewWrapper, el => {
  el.addEventListener("wheel", handleMemoryScroll);
}, { once: true });

onMounted(() => {
  refreshMemoryPanel();
  window.addEventListener("resize", refreshMemoryPanel);
})
onUnmounted(() => {
  memViewWrapper.value?.removeEventListener("wheel", handleMemoryScroll);
  window.removeEventListener("resize", refreshMemoryPanel);

})
onActivated(() => {
  const asmFileName = activeFileStore.path;
  if (asmFileName != null && activeFileStore.lastBuilt > activeFileStore.lastLoaded) {
    const objFileName = asmFileName.replace(/\.asm$/, ".obj");
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

async function dropFile(e: DragEvent) {
  const file = e.dataTransfer.files[0];
  if (file?.name.toLowerCase().endsWith("obj") && !lc3.isSimRunning()) {
    openFile(fs.getPath(file));
  }
}
async function openFile(path: string | undefined = undefined) {
  let selectedFiles: string[] = [];
  if (!path) {
    const result = await dialog.showModal("open", {
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
  // pause lc3 if running
  lc3.pause();

  lc3.clearInput();
  // clear output on file (re)load
  if (settings.clear_out_on_reload) {
    clearConsoleOutput();
  }

  // load object file
  // and check for load failure (i.e., if file is malformed)
  let success;
  try {
    lc3.loadObjectFile(path);
    success = true;
  } catch {
    success = false;
  }

  // If successful, set up initialization steps
  if (success) {
    lastLoadedFile = path;
    memView.value.start = lc3.getRegValue("pc");
    memView.value.symTable = lc3.getCurrSymTable();
  }
  updateUI();
  isSnackBarVisible.value = success;
}
function reloadFile() {
  loadFile(lastLoadedFile);
  updateUI();
}
function toggleSimulator(runKind: "in" | "out" | "over" | "run") {
  if (!sim.value.running) {
    sim.value.running = true;

    startPollIO();

    return new Promise<void>((resolve, reject) => {
      const callback = (error: Error) => {
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
        // statically assert no other branches exist:
        runKind satisfies never;
      }
    });
  } else {
    endSimulation(false);
  }
}
function reinitializeMachine() {
  lc3.reinitializeMachine();
  lc3.clearInput();
  clearConsoleOutput();
  updateUI();
}
function randomizeMachine() {
  lc3.randomizeMachine();
  lc3.clearInput();
  clearConsoleOutput();
  updateUI();
}

function startPollIO() {
  if (typeof pollOutputHandle !== "number") {
    pollOutputHandle = setInterval(() => {
      updateConsole();
      updateTimer();
    }, 50);
  }
}
function stopPollOutput() {
  clearInterval(pollOutputHandle);
  pollOutputHandle = null;
}

function endSimulation(jumpToPC_: boolean) {
  stopPollOutput();

  if (sim.value.running) {
    sim.value.running = false;
    lc3.pause();
    updateUI(true);

    sim.value.regs[9].value = lc3.getRegValue("pc");
    if (jumpToPC_) jumpToPC(false);
  }
}
function clearConsoleOutput() {
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
    const code = key.charCodeAt(0);
    if (code > 64 && code < 128 && e.ctrlKey) {
      key = String.fromCharCode(code & 0x1F);
    } 
    lc3.addInput(key);
  }

  e.preventDefault(); // for TAB, etc.
}

async function openRegContextMenu(item: RegDataRow) {
  if (lc3.isSimRunning()) return;

  const output = await dialog.showModal("menu", ["Jump", "Copy Hex", "Copy Decimal"]);
  switch (output) {
    case 0:
      jumpToMemView(item.value);
      break;
    
    // These two functions are actually pretty useless.
    // They're only here so "Jump" isn't by itself.
    case 1:
      navigator.clipboard.writeText(toHex(item.value));
      break;
    case 2:
      navigator.clipboard.writeText(String(toFormattedDec(item.value)));
      break;
  }
}
async function openMemContextMenu(item: MemDataRow) {
  if (lc3.isSimRunning()) return;

  const options = ["Jump to Address"];

  const hasLabel = !!item.label;
  const hasInstr = typeof lc3.getAddrSourceRange(item.addr) !== "undefined";
  if (hasLabel && hasInstr) {
    options.push("Jump to Source (Label)", "Jump to Source (Instruction)");
  } else if (hasLabel || hasInstr) {
    options.push("Jump to Source");
  }

  options.push("Copy Hex", "Copy Decimal");
  const output = await dialog.showModal("menu", options);
  switch (options[output]) {
    case "Jump to Address":
      jumpToMemView(item.value);
      break;
    
    case "Jump to Source":
      jumpToSource(item.label || item.addr);
      break;
    case "Jump to Source (Label)":
      jumpToSource(item.label);
      break;
    case "Jump to Source (Instruction)":
      jumpToSource(item.addr);
      break;

    // These two functions are actually pretty useless.
    // They're only here so "Jump" isn't by itself.
    case "Copy Hex":
      navigator.clipboard.writeText(toHex(item.value));
      break;
    case "Copy Decimal":
      navigator.clipboard.writeText(String(toFormattedDec(item.value)));
      break;
  }
}
function setDataValue(dataCell: RegDataRow, type: "reg", rules: ValidationRule[]): void;
function setDataValue(dataCell: MemDataRow, type: "mem", rules: ValidationRule[]): void;
function setDataValue(dataCell: RegDataRow | MemDataRow, type: "reg" | "mem", rules: ValidationRule[]) {
  const value = editValue.value;
  const validated = rules.every(r => r(value) === true);
  
  // Validation failed, so ignore set
  if (!validated) {
    if (type === "reg" && "name" in dataCell) {
      dataCell.value = lc3.getRegValue(dataCell.name);
    } else if (type === "mem" && "addr" in dataCell) {
      dataCell.value = lc3.getMemValue(dataCell.addr);
    }
    return;
  }

  dataCell.value = toUint16(parseInputString(value));
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
    for (const reg of sim.value.regs) {
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
  const updates: number[] = lc3.takeMemChanges();
  for (let i = 0; i < memView.value.data.length; i++) {
    const addr = toUint16(memView.value.start + i);
    const dataLine = memView.value.data[i];

    dataLine.addr = addr;
    dataLine.value = lc3.getMemValue(addr);
    dataLine.line = lc3.getMemLine(addr);
    // show label using symbol table
    dataLine.label = memView.value.symTable[addr]?.toUpperCase() ?? "";
  
    dataLine.flash = false;
    dataLine.updated = false;
    if (showUpdates && updates.includes(addr)) {
      dataLine.flash = true;
      setTimeout(() => {
        dataLine.flash = false;
        dataLine.updated = true;
      }, 250);
    }
  }

  updateConsole();
  updateTimer();
}
function updateConsole() {
  consoleStr.value += lc3.getAndClearOutput();
}
function updateTimer() {
  if (sim.value.timer.enabled) {
    sim.value.timer.remaining = lc3.getTimerRemaining();
  }
}
function toggleBreakpoint(addr: number) {
  const idx = sim.value.breakpoints.indexOf(addr);

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
      // statically assert no other branches exist:
      location satisfies never;
    }

    if (typeof span !== "undefined") {
      const [slno, scno, elno, ecno] = span;
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
  const match = jumpToLocInput.value.match(/^(?:0?[xX])?([0-9A-Fa-f]+)$/);
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
  const pc = toUint16(sim.value.regs[9].value);
  const memViewStart = memView.value.start;
  const memViewEnd = memViewStart + memView.value.data.length;
  
  const pcInView = memViewStart <= pc && pc < memViewEnd;
  if (jumpIfInView || !pcInView) jumpToMemView(pc);
}

// Timer functions
function setTimerStatus() {
  lc3.setTimerStatus(sim.value.timer.enabled);
  resetTimer();
}
function resetTimer() {
  lc3.resetTimer();
  updateUI();
}
function resetTimerInputs() {
  timerInputs.value = {
    vect: "x" + lc3.getTimerVect().toString(16).padStart(2, "0"),
    priority: String(lc3.getTimerPriority()),
    max: String(lc3.getTimerMax())
  }
}
async function setTimerProperty(event: SubmitEvent & Promise<{valid: boolean}>, prop: keyof typeof timerInputs.value) {
  const { valid } = await event;
  if (!valid) return;

  if (prop === "vect") {
    const intValue = parseInputString(timerInputs.value[prop]) & 0xFF;
    lc3.setTimerVect(intValue);
    sim.value.timer[prop] = intValue;
  } else if (prop === "priority") {
    const intValue = parseInputString(timerInputs.value[prop]);
    lc3.setTimerPriority(intValue);
    sim.value.timer[prop] = intValue;
  } else if (prop === "max") {
    const intValue = parseInputString(timerInputs.value[prop]);
    lc3.setTimerMax(intValue);
    sim.value.timer[prop] = intValue;
    resetTimer();
  } else {
    prop satisfies never;
  }
}
// Helper functions
function psrToCC(psr: number) {
  const cc = psr & 0b111;
  switch (cc) {
    case 0b100: return "N"
    case 0b010: return "Z"
    case 0b001: return "P"
    default: return "?"
  }
}
function toHex(value: number) {
  const hex = toUint16(value).toString(16).toUpperCase();
  return `x${hex.padStart(4, "0")}`;
}
function toFormattedDec(value: number) {
  if (settings.numbers === "signed") {
    return toInt16(value);
  } else if (settings.numbers === "unsigned") {
    return toUint16(value);
  } else {
    // statically assert no other branches exist:
    settings.numbers satisfies never;
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
.h-limit {
  height: calc(100vh - 90px);
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
  transition: 
    background-color 0.25s ease-in-out,
    color 0.25s ease-in-out
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
.sim-data-table tbody td {
  /* Hide overlong labels + instrs */
  overflow: hidden;
  white-space: nowrap;
}
.row-update-flash {
  background-color: #fff700a0;
}
.row-updated {
  background-color: #fff70038;
}
.sim-top:not(.reduce-flashing) .row-disabled {
  color: gray;
  background-color: lightgrey !important;
}
.sim-top.reduce-flashing .row-disabled {
  color: gray;
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
  transition: color 0.1s, font-size 0.2s;
}
.data-cell-btn:deep(button) {
  background-color: transparent;
  /* Force height of buttons to be smaller than the height of each row */
  height: calc(var(--v-table-row-height) - 1);
}
.data-cell-num {
  text-align: right !important;
}

tr:not(.row-disabled) .clickable {
  cursor: pointer;
}
/* Console styles */
#console-wrapper {
  display: flex;
  flex-direction: column;
  flex: 1;
  /* Prevents console from overflowing past page */
  min-height: 0;
}

/* A header with icons around it */
.header-bar {
  display: grid;
  grid-template-columns: 50px auto 50px;
  grid-template-rows: 100%;
  justify-items: center;
  align-items: center;
  overflow: hidden;
}
.header-bar-title {
  text-align: center;
  padding-bottom: 5px;

  grid-column: 2;
  grid-row: 1;
}
.header-bar-right {
  grid-column: 3;
  grid-row: 1;
}

/* Memory view styles */
.row-curr-pc {
  background-color: #008cff4d;
}

tr:not(.row-disabled) .breakpoint-icon:hover {
  color: red !important;
}

tr:not(.row-disabled) .pc-icon:hover {
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

.contents {
  display: contents;
}
</style>
