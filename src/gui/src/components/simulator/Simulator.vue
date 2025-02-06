<script setup lang="ts">
import { useActiveFileStore } from '../../store/active_file';
import { useSettingsStore } from '../../store/settings';
// Vue stuff
import { computed, nextTick, onActivated, onMounted, onUnmounted, ref, useTemplateRef, watch } from 'vue';
import { useRouter } from 'vue-router';
import "vuetify/components";
//
import Console from '../Console.vue';
import { useToast } from 'primevue';

const { lc3, dialog, fs } = window.api;

const settings = useSettingsStore();
const activeFileStore = useActiveFileStore();
const router = useRouter();
const toast = useToast();
const timerPopover = useTemplateRef("timerPopover");
const hexPopover = useTemplateRef("hexPopover");
const decPopover = useTemplateRef("decPopover");

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

const consoleStr = ref("");
const jumpToLocInput = ref("");
const timerInputs = ref({
  vect: "x81",
  priority: 4,
  max: 50
});

const timerRemBadgeShow = computed(() => sim.value.timer.enabled && !sim.value.timer.hide_badge && (sim.value.running || sim.value.timer.remaining != 0));
const timerBtnVariant = computed(() => {
  if (sim.value.timer.enabled) {
    if (!sim.value.running && sim.value.timer.remaining == 0) return null;
    return "outlined";
  }
  return "text";
});
const timerBtnColor = computed(() => sim.value.timer.enabled ? "primary" : "secondary");
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

function showFileLoadedToast() {
  toast.add({ severity: 'contrast', summary: 'Object File Loaded!', life: 2500 });
}
function showEditPopover(popover: typeof timerPopover["value"], e: Event) {
  editValue.value = (e.target as HTMLElement).textContent;

  popover.hide();
  nextTick(() => {
    popover.show(e);
  })
}
function refreshMemoryPanel() {
  const rowWidth = Math.min(memViewWrapper.value.querySelector("tr")?.offsetHeight ?? 0, 25);

  memView.value.data = Array.from(
    { length: Math.floor((window.innerHeight) / rowWidth) - 10},
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
    showFileLoadedToast();
  }
  updateUI();
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
    priority: lc3.getTimerPriority(),
    max: lc3.getTimerMax()
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
    const intValue = timerInputs.value[prop];
    lc3.setTimerPriority(intValue);
    sim.value.timer[prop] = intValue;
  } else if (prop === "max") {
    const intValue = timerInputs.value[prop];
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
  } else if (item.name.startsWith("r") && 20 <= item.value && item.value <= 127) {
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

<template>
  <div 
    class="sim-top"
    :class="{
      'reduce-flashing': settings.reduce_flashing
    }"
  >
    <nav-menu>
      <!-- File buttons -->
      <nav-icon
        label="Open File"
        @click="openFile()"
      >
        <MdiFolderOpen />
      </nav-icon>
      <nav-icon
        :label="sim.running ? 'Pause' : 'Play'"
        @click="toggleSimulator('run')"
      >
        <MdiPause v-if="sim.running" />
        <MdiPlay v-else />
      </nav-icon>
      <nav-icon
        label="Reload Object Files"
        @click="reloadFile()"
      >
        <MdiRefresh />
      </nav-icon>

      <!-- Debug -->
      <Divider class="my-0" />
      <nav-icon
        label="Step Over"
        @click="toggleSimulator('over')"
      >
        <MdiDebugStepOver />
      </nav-icon>
      <nav-icon
        label="Step In"
        @click="toggleSimulator('in')"
      >
        <MdiDebugStepInto />
      </nav-icon>
      <nav-icon
        label="Step Out"
        @click="toggleSimulator('out')"
      >
        <MdiDebugStepOut />
      </nav-icon>

      <!-- Machine -->
      <Divider class="my-0" />
      <nav-icon
        label="Reinitialize Machine"
        @click="reinitializeMachine()"
      >
        <MdiPower />
      </nav-icon>
      <nav-icon
        label="Randomize Machine"
        @click="randomizeMachine()"
      >
        <MdiShuffle />
      </nav-icon>
    </nav-menu>
    <!-- Toast popup -->
    <Toast position="top-center">
      <template #container="{ message, closeCallback }">
        <div class="flex p-2 items-center">
          <div class="px-2 flex-1">
            {{ message.summary }}
          </div>
          <Button
            icon="pi"
            variant="text"
            rounded
            severity="danger"
            @click="closeCallback"
          >
            <MdiClose />
          </Button>
        </div>
      </template>
    </Toast>

    <!-- Edit value popovers -->
    <!-- TODO: Validate (rules.hex, rules.size16bit) -->
    <Popover
      v-if="!sim.running"
      ref="hexPopover"
    >
      <div>
        <InputNumber
          size="small"
          placeholder="Hex Value"
        />
      </div>
    </Popover>
    <!-- TODO: Validate (rules.dec, rules.size16bit) -->
    <Popover
      v-if="!sim.running"
      ref="decPopover"
    >
      <div>
        <InputNumber
          size="small"
          placeholder="Dec Value"
        />
      </div>
    </Popover>
    <!-- Main editor content -->
    <main
      class="contents"
      @drop.prevent="dropFile"
      @dragover.prevent
    >
      <div class="grid grid-cols-[1fr_2fr] w-full gap-4 p-4 pt-2">
        <div class="flex flex-col gap-1">
          <div class="header-bar">
            <div />
            <h3 class="header-bar-title">
              Registers
            </h3>
          </div>
          <div>
            <table class="sim-data-table">
              <colgroup>
                <col style="width: 20%">
                <col style="width: 20%">
                <col style="width: 20%">
                <col style="width: 40%">
              </colgroup>
              <thead>
                <tr>
                  <th class="data-cell-text">
                    Registers
                  </th>
                  <th class="data-cell-num">
                    Hex
                  </th>
                  <th class="data-cell-num">
                    Decimal
                  </th>
                  <th class="data-cell-text">
                    ASCII / Misc
                  </th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="item of sim.regs"
                  :key="item.name"
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
                    @click="(e) => showEditPopover(hexPopover, e)"
                  >
                    <span>
                      {{ toHex(item.value) }}
                    </span>
                  </td>
                  <td
                    class="data-cell-num clickable"
                    @click="(e) => showEditPopover(decPopover, e)"
                  >
                    <span>{{ toFormattedDec(item.value) }}</span>
                  </td>
                  <td class="data-cell-text">
                    <span>{{ regLabel(item) }}</span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
          <div class="flex flex-col flex-1">
            <div class="header-bar">
              <div />
              <h3 class="header-bar-title">
                Console (click to focus)
              </h3>
              <Button
                v-tooltip.left="'Clear Console'"
                icon="pi"
                rounded
                variant="text"
                @click="clearConsoleOutput()"
              >
                <MdiDelete class="text-black dark:text-white" />
              </Button>
            </div>
            <console 
              v-model="consoleStr"
              float="bottom"
              show-focus
              show-cursor
              @keydown="handleConsoleInput"
            />
          </div>
        </div>
        <div class="flex flex-col gap-1">
          <div class="header-bar">
            <div />
            <h3 class="header-bar-title">
              Memory
            </h3>
            <OverlayBadge
              severity="secondary"
              :value="sim.timer.remaining || ''"
              :class="{ 'hide-badge': !timerRemBadgeShow }"
            >
              <Button
                v-tooltip.left="'Configure Timer Interrupt'"
                :variant="timerBtnVariant"
                icon="pi"
                rounded
                :severity="timerBtnColor"
                @click="e => {
                  resetTimerInputs();
                  timerPopover?.toggle(e);
                }"
              >
                <MdiTimer />
              </Button>
            </OverlayBadge>
            <Popover
              v-if="!sim.running"
              ref="timerPopover"
            >
              <div class="popover-menu">
                <div>
                  <label>
                    <span>Enable timer interrupt</span>
                    <ToggleSwitch v-model="sim.timer.enabled" />
                  </label>
                  <label>
                    <span>Hide timer badge</span>
                    <ToggleSwitch
                      v-model="sim.timer.hide_badge"
                      :disabled="!sim.timer.enabled"
                    />
                  </label>
                </div>
                <Divider />
                <div>
                  <!-- TODO: impl form submission -->
                  <label>
                    <span>Vector</span>
                    <!-- TODO: Validate (rules.hex, rules.size8bit) -->
                    <InputText
                      v-model="timerInputs.vect"
                      :disabled="!sim.timer.enabled"
                      class="w-24"
                    />
                  </label>
                  <label>
                    <span>Priority</span>
                    <InputNumber
                      v-model="timerInputs.priority"
                      :use-grouping="false"
                      :disabled="!sim.timer.enabled"
                      :min="0"
                      :max="7"
                      input-class="w-24"
                    />
                  </label>
                  <label>
                    <span>Repeat</span>
                    <InputNumber
                      v-model="timerInputs.max"
                      :use-grouping="false"
                      :disabled="!sim.timer.enabled"
                      :min="0"
                      :max="2 ** 31 - 1"
                      input-class="w-24"
                    />
                  </label>
                </div>
                <Divider />
                <div>
                  Interrupt activates in {{ sim.timer.enabled ? sim.timer.remaining : "-" }} instruction{{ sim.timer.remaining !== 1 ? 's' : '' }}
                </div>
              </div>
            </Popover>
          </div>
          <table
            ref="memViewWrapper"
            class="sim-data-table" 
          >
            <colgroup>
              <col style="width: 2em">
              <col style="width: 2em">
              <col style="width: 10%">
              <col style="width: 10%">
              <col style="width: 10%">
              <col style="width: 15%">
              <col style="width: 45%">
            </colgroup>
            <thead>
              <tr>
                <th class="data-cell-btn">
                  BP
                </th>
                <th class="data-cell-btn">
                  PC
                </th>
                <th class="data-cell-num">
                  Address
                </th>
                <th class="data-cell-num">
                  Hex
                </th>
                <th class="data-cell-num">
                  Decimal
                </th>
                <th class="data-cell-text">
                  Label
                </th>
                <th class="data-cell-text">
                  Instructions
                </th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="item of memView.data"
                :key="item.addr"
                :class="{
                  'row-update-flash': item.flash,
                  'row-updated': item.updated,
                  'row-disabled': sim.running,
                  'row-curr-pc': isPCAt(item.addr)
                }"
                @contextmenu="openMemContextMenu(item)"
              >
                <td class="data-cell-btn">
                  <div class="flex items-center">
                    <button @click="toggleBreakpoint(item.addr)">
                      <MdiAlertOctagon 
                        class="breakpoint-icon"
                        :class="{ 'icon-active': isBreakpointAt(item.addr) }"
                      />
                    </button>
                  </div>
                </td>
                <td class="data-cell-btn">
                  <div class="flex items-center">
                    <button @click="setPC(item.addr)">
                      <MdiPlay
                        class="pc-icon"
                        :class="{ 'icon-active': isPCAt(item.addr) }"
                      />
                    </button>
                  </div>
                </td>
                <td class="data-cell-num">
                  <strong>{{ toHex(item.addr) }}</strong>
                </td>
                <td
                  class="data-cell-num clickable"
                  @click="(e) => showEditPopover(hexPopover, e)"
                >
                  <span>{{ toHex(item.value) }}</span>
                </td>
                <td
                  class="data-cell-num clickable"
                  @click="(e) => showEditPopover(decPopover, e)"
                >
                  <span>{{ toFormattedDec(item.value) }}</span>
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
            </tbody>
          </table>
          <div class="flex items-center justify-between grow">
            <div>
              <!-- TODO: add form functionality: jumpToMemViewStr() -->
              <InputText placeholder="Jump to Location" />
            </div>
            <div class="flex gap-1">
              <Button
                v-tooltip.top="'Jump to ' + toHex(toUint16(memView.start - memView.data.length))"
                icon="pi"
                rounded
                severity="secondary"
                @click="jumpToPrevMemView()"
              >
                <MdiChevronDoubleLeft />
              </Button>
              <Button
                v-tooltip.top="'Jump to ' + toHex(toUint16(memView.start - 5))"
                icon="pi"
                rounded
                severity="secondary"
                @click="jumpToPartMemView(-5)"
              >
                <MdiChevronLeft />
              </Button>
              <Button
                v-tooltip.top="'Jump to PC'"
                icon="pi"
                severity="secondary"
                @click="jumpToPC(true)"
              >
                <MdiHome />
              </Button>
              <Button
                v-tooltip.top="'Jump to ' + toHex(toUint16(memView.start + 5))"
                icon="pi"
                rounded
                severity="secondary"
                @click="jumpToPartMemView(+5)"
              >
                <MdiChevronRight />
              </Button>
              <Button
                v-tooltip.top="'Jump to ' + toHex(toUint16(memView.start + memView.data.length))"
                icon="pi"
                rounded
                severity="secondary"
                @click="jumpToNextMemView()"
              >
                <MdiChevronDoubleRight />
              </Button>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>
  

<style scoped lang="postcss">
.sim-data-table tbody tr {
  @apply transition duration-300 ease-in-out;
}

.sim-top:not(.reduce-flashing) .row-disabled {
  @apply text-surface-500 bg-surface-300 dark:bg-surface-700;

}
.sim-top.reduce-flashing .row-disabled {
  @apply text-surface-500;
}

.sim-data-table {
  @apply border shadow dark:border-surface-800 table-fixed w-full;
}
.sim-data-table th, .sim-data-table td {
  /* Add padding to all cells */
  /* Hide overlong labels */
  @apply px-2 overflow-hidden whitespace-nowrap;
}
.sim-data-table thead tr {
  @apply bg-surface-400 dark:bg-surface-600;
}
.sim-data-table tr {
  @apply border-b border-surface-200 dark:border-surface-800;
}
.sim-data-table tbody tr:hover {
  @apply bg-surface-500/25;
}

.sim-data-table tbody .data-cell-text, .sim-data-table tbody .data-cell-num {
  @apply font-mono;
}
.data-cell-text {
  @apply text-left;
}
.data-cell-btn {
  @apply text-center;
}
.data-cell-btn button {
  @apply h-6 w-6 flex justify-center items-center transition;
}
.data-cell-num {
  @apply text-right;
}
.row-update-flash {
  background-color: #fff700a0;
}
.row-updated {
  background-color: #fff70038;
}

tr:not(.row-disabled) .clickable {
  cursor: pointer;
}

/* A header with icons around it */
.header-bar {
  display: grid;
  grid-template-columns: 50px auto 50px;
  grid-template-rows: 100%;
  justify-items: center;
  align-items: center;
  @apply min-h-10;
}
.header-bar-title {
  @apply font-bold text-lg text-center;
}

/* Memory view styles */
.row-curr-pc {
  background-color: #008cff4d;
}

tr .breakpoint-icon, tr .pc-icon {
  @apply transition;
}
tr .breakpoint-icon:not(.icon-active), tr .pc-icon:not(.icon-active) {
  @apply text-surface-400 scale-[80%];
}
tr .breakpoint-icon.icon-active {
  @apply text-red-500;
}
tr .pc-icon.icon-active {
  @apply text-blue-500;
}

tr:not(.row-disabled) .breakpoint-icon:hover {
  @apply text-red-500;
}

tr:not(.row-disabled) .pc-icon:hover {
  @apply text-blue-500;
}

.popover-menu > div {
  @apply flex flex-col gap-3;
}
.popover-menu > div > label {
  @apply flex justify-between items-center gap-2;
}
.p-overlaybadge :deep(.p-badge) {
  @apply transition;
}
.p-overlaybadge.hide-badge :deep(.p-badge) {
  @apply opacity-0;
}
</style>
