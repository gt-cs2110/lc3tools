<script setup lang="ts">
import { useActiveFileStore } from '../../store/active_file';
import { useSettingsStore } from '../../store/settings';
// Vue stuff
import { computed, nextTick, onActivated, onMounted, onUnmounted, ref, useTemplateRef, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useToast } from 'primevue';
import { FormResolverOptions, FormSubmitEvent } from '@primevue/forms';
//
import Console from '../Console.vue';

const { lc3, dialog, fs } = window.api;

const settings = useSettingsStore();
const activeFileStore = useActiveFileStore();
const router = useRouter();
const toast = useToast();
const timerPopover = useTemplateRef("timerPopover");

const editPopovers = {
  hex: useTemplateRef("hexPopover"),
  dec: useTemplateRef("decPopover")
}
const editInputs = {
  item: undefined as MemDataRow | RegDataRow | undefined,
  input: ref("")
};
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
  breakpoints: [] as { addr: number, enabled: boolean }[],
  running: false,
  timer: {
    enabled: false,
    hide_badge: false,
    vect: 0x81,
    priority: 4,
    remaining: 0,
    max: 0,
  },
  frame_no: 0
})
const memView = ref({
  start: 0x3000,
  data: [{
    addr: 0, value: 0, line: "", label: "", flash: false, updated: false
  }],
  symTable: {} as Record<number, string>
})

const consoleStr = ref("");
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

const panels = ref({
  showConsole: true,
  showDebugger: true
});
const stackDialog = ref({
  show: false,
  frameReg: 5,
  stackReg: 6,
  pushInput: "",
  offset: 0,
  wheelOffset: 0
});

type RegDataRow = typeof sim.value.regs[number];
type MemDataRow = typeof memView.value.data[number];

const rules: Record<string, ValidationRule> = {
  required(value: string) {
    return !!value || "Input cannot be empty";
  },
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

const memViewWrapper = useTemplateRef("memViewWrapper");
watch(memViewWrapper, el => {
  el.addEventListener("wheel", handleMemoryScroll, { passive: true });
}, { once: true });

onMounted(() => {
  refreshMemoryPanel();
  jumpToPC(true);
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
async function showEditPopover(popoverKey: keyof typeof editPopovers, e: Event, item: RegDataRow | MemDataRow) {
  editInputs.item = item;
  if (popoverKey === "dec") {
    editInputs.input.value = toFormattedDec(item.value);
  } else if (popoverKey === "hex") {
    editInputs.input.value = toHex(item.value);
  }

  const popover = editPopovers[popoverKey].value;
  popover.hide();

  await nextTick();
  // Adjust popover to always point to span
  const target = e.target instanceof HTMLTableCellElement ? e.target.firstElementChild : e.target;
  popover.show(e, target);
}
function applyEditInput(e: FormSubmitEvent) {
  if (e.valid) {
    setDataValue(editInputs.item, parseInputString(e.states.input.value));
    for (const popover of Object.values(editPopovers)) {
      popover.value.hide();
    }
  }
}
function validateInput(key: string, input: string, rules: ValidationRule[]): { errors: Record<string, any> } {
  for (const rule of rules) {
    const result = rule(input);
    if (typeof result === "string") {
      return { errors: { [key]: [result] } };
    }
  }

  return { errors: [] };
}
function validateEditInput(e: FormResolverOptions, ...rules: ValidationRule[]) {
  return validateInput("input", e.values.input, rules);
}
function refreshMemoryPanel() {
  const oldLen = memView.value.data.length;
  const newLen = Math.max(0, Math.floor(memViewWrapper.value.parentElement.offsetHeight / 25));

  if (newLen < oldLen) {
    // Truncate if new size is smaller:
    memView.value.data.length = newLen;
  } else {
    // Add elements if new size is larger:
    for (let i = oldLen; i < newLen; i++) {
      memView.value.data.push({
        addr: 0,
        value: 0,
        line: "",
        label: "",
        flash: false,
        updated: false
      });
    }
  }

  updateUI();
}

let scrollInterval: any = undefined;
const scrolling = ref(false);
function handleMemoryScroll(e: WheelEvent) {
  if (!lc3.isSimRunning()) {
    scrolling.value = true; // track scrolling

    memScrollOffset += e.deltaY;
    if (Math.abs(memScrollOffset) > 20) {
      jumpToPartMemView(Math.trunc(memScrollOffset / 20));
      memScrollOffset = 0;
    }

    // untrack scrolling event
    clearInterval(scrollInterval);
    scrollInterval = setInterval(() => {
      scrolling.value = false;
    }, 50);
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
  if (lastLoadedFile == null) {
    openFile();
  } else {
    loadFile(lastLoadedFile);
  }
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
      navigator.clipboard.writeText(toFormattedDec(item.value));
      break;
  }
}
async function openMemContextMenu(item: MemDataRow) {
  if (lc3.isSimRunning()) return;

  const options = [];
  if ((item.value & 0xF800) == 0x4800 /* is JSR */) {
    options.push("Jump to Subroutine");
  }
  options.push("Jump to Address");

  const hasLabel = !!item.label;
  const hasInstr = typeof lc3.getAddrSourceRange(item.addr) !== "undefined";
  if (hasLabel && hasInstr) {
    options.push("View Source (Label)", "View Source (Instruction)");
  } else if (hasLabel || hasInstr) {
    options.push("View Source");
  }

  options.push("Copy Hex", "Copy Decimal");
  const output = await dialog.showModal("menu", options);
  switch (options[output]) {
    case "Jump to Address":
      jumpToMemView(item.value);
      break;
    case "Jump to Subroutine": {
      // v get PC + sext(PCOffset11)
      const addr = item.addr + 1 + ((item.value << (32 - 11)) >> (32 - 11));
      jumpToMemView(addr);
      break;
    }

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
      navigator.clipboard.writeText(toFormattedDec(item.value));
      break;
  }
}
function setDataValue(cell: RegDataRow | MemDataRow, value: number) {
  cell.value = toUint16(value);
  if ("name" in cell) {
    lc3.setRegValue(cell.name, cell.value);
  } else if ("addr" in cell) {
    lc3.setMemValue(cell.addr, cell.value);
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

  // Update breakpoints (in case the engine's breakpoints get desynced):
  sim.value.breakpoints = sim.value.breakpoints.filter(({enabled}) => !enabled)
    .concat(lc3.getBreakpoints().map(addr => ({ addr, enabled: true })))
    .sort((a, b) => a.addr - b.addr);
  // Update frame number:
  sim.value.frame_no = lc3.getFrameNumber();
  // Update these:
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

/**
 * Handles internal breakpoint status when using enabled/disabled.
 * @param addr The address of the breakpoint
 * @param enabled The enabled status of the breakpoint
 */
function setInternalBreakpointStatus(addr: number, enabled: boolean) {
  if (lc3.isSimRunning()) return;

  if (enabled) {
    lc3.setBreakpoint(addr);
  } else {
    lc3.removeBreakpoint(addr);
  }
}
/**
 * Adds a breakpoint and sets it to enabled.
 * @param addr The address of the breakpoint
 */
function addBreakpoint(addr: number) {
  if (lc3.isSimRunning()) return;

  lc3.setBreakpoint(addr);
  // There are more efficient ways of doing this, 
  // but I do not wish to reimplement partition_point currently.
  sim.value.breakpoints.push({addr, enabled: true });
  sim.value.breakpoints.sort((a, b) => a.addr - b.addr);
}
/**
 * Removes a breakpoint. This is different from disabling a breakpoint 
 * because it is completely removed from the UI.
 * @param addr The address of the breakpoint
 * @param idx Optionally, the index of the breakpoint in the breakpoint list. 
 *     If not provided, this is computed.
 */
function removeBreakpoint(addr: number, idx?: number) {
  if (lc3.isSimRunning()) return;
  if (typeof idx === "undefined") {
    idx = sim.value.breakpoints.findIndex(bp => bp.addr == addr);
  }

  lc3.removeBreakpoint(addr);
  sim.value.breakpoints.splice(idx, 1);
}
/**
 * Toggles breakpoint to the next state.
 * - If breakpoint exists and is enabled, this removes the breakpoint.
 * - If breakpoint exists and is disabled, this enables the breakpoint.
 * - If the breakpoint does not exist, this creates the breakpoint and enables it.
 * @param addr The address of the breakpoint
 */
function toggleBreakpoint(addr: number) {
  const idx = sim.value.breakpoints.findIndex(bp => bp.addr == addr);

  if (!lc3.isSimRunning()) {
    if (idx == -1) {
      addBreakpoint(addr);
    } else {
      const bp = sim.value.breakpoints[idx];
      if (!bp.enabled) {
        bp.enabled = true;
        setInternalBreakpointStatus(addr, true);
      } else {
        removeBreakpoint(addr, idx);
      }
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
  return sim.value.breakpoints.some(bp => bp.addr == addr);
}
function isPCAt(addr: number) {
  return addr == sim.value.regs[9].value && !sim.value.running;
}

// Memory view jump functions
function jumpToMemView(newStart: number) {
  memView.value.start = toUint16(newStart);
  updateUI(false, false);
}
function jumpToMemViewStr(input: string) {
  const match = input.match(/^(?:0?[xX])?([0-9A-Fa-f]+)$/);
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
watch(() => sim.value.timer.enabled, enabled => {
  lc3.setTimerStatus(enabled);
  resetTimer();
})
function resetTimer() {
  lc3.resetTimer();
  updateUI();
}
function timerValidator(e: FormResolverOptions) {
  const result = validateInput("vect", e.values.vect, [rules.hex, rules.size8bit]);
  Object.assign(result.errors, {
    priority: typeof e.values.priority != "number" ? ["Argument is required"] : [],
    max: typeof e.values.max != "number" ? ["Argument is required"] : [],
  });

  return result;
}
function updateTimerProperties(e: FormSubmitEvent) {
  if (e.valid) {
    // Set vect:
    const vect: number = parseInputString(e.states.vect.value) & 0xFF;
    if (sim.value.timer.vect != vect) {
      lc3.setTimerVect(vect);
      sim.value.timer.vect = vect;
    }

    // Set priority:
    const priority: number = e.states.priority.value;
    if (sim.value.timer.priority != priority) {
      lc3.setTimerPriority(priority);
      sim.value.timer.priority = priority;
    }
    
    // Set max/repeat
    const max: number = e.states.max.value;
    if (sim.value.timer.max != max) {
      lc3.setTimerMax(max);
      sim.value.timer.max = max;
      resetTimer();
    }
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
function toHex(value: number): string {
  const hex = toUint16(value).toString(16).toUpperCase();
  return `x${hex.padStart(4, "0")}`;
}
function toFormattedDec(value: number): string {
  if (settings.numbers === "signed") {
    return String(toInt16(value));
  } else if (settings.numbers === "unsigned") {
    return String(toUint16(value));
  } else {
    // statically assert no other branches exist:
    settings.numbers satisfies never;
  }
}
function parseInputString(value: string) {
  if (value.startsWith("x") || value.startsWith("X")) value = "0" + value;
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
        <MdiPause v-show="sim.running" />
        <MdiPlay v-show="!sim.running" />
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
      <Divider class="my-0" />
      <nav-icon
        label="Console"
        :toggle="panels.showConsole"
        @click="panels.showConsole = !panels.showConsole"
      >
        <MdiConsole />
      </nav-icon>
      <nav-icon
        label="Debugger"
        :toggle="panels.showDebugger"
        @click="panels.showDebugger = !panels.showDebugger"
      >
        <MdiBug />
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
    <Popover
      v-if="!sim.running"
      ref="hexPopover"
    >
      <div>
        <Form
          v-slot="$form"
          :initial-values="{ input: editInputs.input.value }"
          :resolver="e => validateEditInput(e, rules.hex, rules.size16bit)"
          @submit="e => applyEditInput(e)"
        >
          <div class="flex flex-col gap-1">
            <IftaLabel>
              <InputText
                id="hex-popover-input"
                name="input"
                size="small"
                :invalid="$form.input?.invalid"
              />
              <label for="hex-popover-input">Hex Value</label>
            </IftaLabel>
            <Message
              v-if="$form.input?.invalid"
              severity="error"
              variant="simple"
              size="small"
            >
              {{ $form.input?.error }}
            </Message>
          </div>
        </Form>
      </div>
    </Popover>
    <Popover
      v-if="!sim.running"
      ref="decPopover"
    >
      <div>
        <Form
          v-slot="$form"
          :initial-values="{ input: editInputs.input.value }"
          :resolver="e => validateEditInput(e, rules.dec, rules.size16bit)"
          @submit="e => applyEditInput(e)"
        >
          <div class="flex flex-col gap-1">
            <IftaLabel>
              <InputText
                id="dec-popover-input"
                name="input"
                size="small"
                :invalid="$form.input?.invalid"
              />
              <label for="dec-popover-input">Decimal Value</label>
            </IftaLabel>
            <Message
              v-if="$form.input?.invalid"
              severity="error"
              variant="simple"
              size="small"
            >
              {{ $form.input?.error }}
            </Message>
          </div>
        </Form>
      </div>
    </Popover>
    <!-- Main editor content -->
    <main
      class="contents"
      @drop.prevent="dropFile"
      @dragover.prevent
    >
      <div class="grid grid-cols-[1fr_2fr] grid-rows-1 w-full h-full gap-4 p-4 pt-2">
        <div class="flex flex-col gap-1 min-h-0">
          <div class="header-bar">
            <div />
            <h3 class="header-bar-title">
              Registers
            </h3>
          </div>
          <div class="rounded">
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
                  class="even:bg-surface-elevated-0"
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
                    @click="e => showEditPopover('hex', e, item)"
                  >
                    <span>
                      {{ toHex(item.value) }}
                    </span>
                  </td>
                  <td
                    class="data-cell-num clickable"
                    @click="e => showEditPopover('dec', e, item)"
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
          <div
            v-if="panels.showConsole"
            class="flex flex-col flex-1 min-h-0"
          >
            <div class="header-bar">
              <div />
              <h3 class="header-bar-title">
                Console
              </h3>
              <Button
                v-tooltip.left="'Clear Console'"
                icon="pi"
                rounded
                variant="text"
                severity="secondary"
                @click="clearConsoleOutput()"
              >
                <MdiDelete />
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
          <div
            v-if="panels.showDebugger"
            class="flex flex-col flex-1 gap-1 min-h-0"
          >
            <div class="header-bar">
              <div />
              <h3 class="header-bar-title">
                Debugger
              </h3>
              <div />
            </div>
            <div>
              <!-- TODO: Add functionality -->
              <div class="flex rounded bg-surface-elevated-1 border border-surface">
                <Button
                  v-tooltip.top="'Step Over'"
                  icon="pi"
                  variant="text"
                  severity="info"
                  rounded
                  label="Step Over"
                  @click="toggleSimulator('over')"
                >
                  <MdiDebugStepOver />
                </Button>
                <Button
                  v-tooltip.top="'Step In'"
                  icon="pi"
                  variant="text"
                  severity="info"
                  rounded
                  label="Step In"
                  @click="toggleSimulator('in')"
                >
                  <MdiDebugStepInto />
                </Button>
                <Button
                  v-tooltip.top="'Step Out'"
                  icon="pi"
                  variant="text"
                  severity="info"
                  rounded
                  label="Step Out"
                  @click="toggleSimulator('out')"
                >
                  <MdiDebugStepOut />
                </Button>
                <div class="flex-1" />
                <div class="flex items-center">
                  <Badge
                    v-tooltip.top="sim.frame_no > 0 ? 'Frame Count' : ''"
                    :value="sim.frame_no"
                    :class="{ 'hide-badge': sim.frame_no <= 0 }"
                    severity="info"
                  />
                </div>
                <Button
                  v-tooltip.top="'Adjust Stack'"
                  icon="pi"
                  variant="text"
                  severity="info"
                  rounded
                  label="Adjust Stack"
                  :disabled="sim.running"
                  @click="stackDialog.show = true"
                >
                  <MdiViewAgenda />
                </Button>
              </div>
            </div>
            <Dialog
              v-model:visible="stackDialog.show"
              modal
              header="Adjust Stack"
              dismissable-mask
            >
              <div
                v-if="stackDialog.show"
                class="grid grid-cols-[1fr_auto] grid-rows-1 gap-3"
              >
                <div class="grid grid-cols-[1fr_1fr_6em] shadow border border-surface gap-x-2">
                  <div class="grid grid-cols-subgrid col-span-3 border-t last:border-b border-surface px-2 bg-table-header font-bold">
                    <div class="text-right">
                      Addr
                    </div>
                    <div class="text-right">
                      Hex
                    </div>
                    <div class="text-right">
                      Dec
                    </div>
                  </div>
                  <div
                    v-for="(addr, i) in Array.from({ length: 15 }, (_, i) => sim.regs[stackDialog.stackReg].value + stackDialog.offset + i)"
                    :key="i"
                    class="grid grid-cols-subgrid col-span-3 border-t last:border-b font-mono px-2 border-surface"
                    :class="{
                      // Note that this doesn't update based on address when the grid moves.
                      // This has been intentionally omitted because it is hard to read.
                      'even:bg-surface-elevated-1': stackDialog.offset + i < 0,
                      'odd:bg-stack-lo even:bg-stack-hi': stackDialog.offset + i >= 0,
                    }"
                    @wheel.passive="e => {
                      if (!lc3.isSimRunning()) {
                        stackDialog.wheelOffset += e.deltaY;
                        if (Math.abs(stackDialog.wheelOffset) > 20) {
                          stackDialog.offset += Math.trunc(stackDialog.wheelOffset / 20);
                          stackDialog.wheelOffset = 0;
                        }
                      }
                    }"
                  >
                    <div
                      v-tooltip.top="
                        addr == sim.regs[stackDialog.stackReg].value ? 
                          'Stack Pointer' : 
                          addr == sim.regs[stackDialog.frameReg].value ? 
                            'Frame Pointer' :
                            ''
                      "
                      class="text-right"
                      :class="{'underline': [sim.regs[stackDialog.frameReg].value, sim.regs[stackDialog.stackReg].value].includes(addr) }"
                    >
                      {{ toHex(addr) }}
                    </div>
                    <div class="text-right">
                      {{ toHex(lc3.getMemValue(addr)) }}
                    </div>
                    <div class="text-right">
                      {{ toFormattedDec(lc3.getMemValue(addr)) }}
                    </div>
                  </div>
                </div>
                <div class="flex flex-col gap-1">
                  <div class="flex items-center justify-center gap-1">
                    <Button
                      v-tooltip.bottom="'Up'"
                      icon="pi"
                      severity="secondary"
                      rounded
                      @click="stackDialog.offset--"
                    >
                      <MdiChevronUp />
                    </Button>
                    <Button
                      v-tooltip.bottom="'Home'"
                      icon="pi"
                      severity="secondary"
                      @click="stackDialog.offset = 0"
                    >
                      <MdiHome />
                    </Button>
                    <Button
                      v-tooltip.bottom="'Down'"
                      icon="pi"
                      severity="secondary"
                      rounded
                      @click="stackDialog.offset++"
                    >
                      <MdiChevronDown />
                    </Button>
                  </div>
                  <Divider class="my-1" />
                  <label class="flex justify-between items-center gap-2">
                    <span>Frame Pointer:</span>
                    <div>
                      R<InputNumber
                        v-model="stackDialog.frameReg"
                        :min="0"
                        :max="7"
                        input-class="w-12"
                        size="small"
                      />
                    </div>
                  </label>
                  <label class="flex justify-between items-center gap-2">
                    <span>Stack Pointer:</span>
                    <div>
                      R<InputNumber
                        v-model="stackDialog.stackReg"
                        :min="0"
                        :max="7"
                        input-class="w-12"
                        size="small"
                      />
                    </div>
                  </label>
                  <Form
                    v-slot="$form"
                    :resolver="e => validateEditInput(e, rules.required, rules.size16bit)"
                    :validate-on-value-update="false"
                    @submit="(e) => {
                      if (e.valid) {
                        const reg = sim.regs[stackDialog.stackReg];
                        const value = parseInputString(e.states.input.value);
                        if (typeof value === 'number') {
                          lc3.setMemValue(reg.value - 1, value);
                          setDataValue(reg, reg.value - 1);
                        }
                        if (-15 + 1 < stackDialog.offset && stackDialog.offset < 0) {
                          stackDialog.offset++;
                        } else {
                          stackDialog.offset = 0;
                        }
                      }
                    }"
                  >
                    <div class="flex items-center">
                      <InputText
                        name="input"
                        placeholder="x2110"
                        class="w-24"
                        size="small"
                        :invalid="$form.input?.invalid"
                      />
                      <Button
                        v-tooltip.bottom="'Push'"
                        icon="pi"
                        variant="text"
                        severity="secondary"
                        rounded
                        type="submit"
                      >
                        <MdiPlus />
                      </Button>
                      <Button
                        v-tooltip.bottom="'Pop'"
                        icon="pi"
                        variant="text"
                        severity="secondary"
                        rounded
                        @click="() => {
                          const reg = sim.regs[stackDialog.stackReg];
                          setDataValue(reg, reg.value + 1);
                          if (-15 + 1 < stackDialog.offset && stackDialog.offset < 0) {
                            stackDialog.offset--;
                          } else {
                            stackDialog.offset = 0;
                          }
                        }"
                      >
                        <MdiMinus />
                      </Button>
                    </div>
                    <Message
                      v-if="$form.input?.invalid"
                      severity="error"
                      variant="simple"
                      size="small"
                      class="w-40"
                    >
                      {{ $form.input?.error }}
                    </Message>
                  </Form>
                </div>
              </div>
            </Dialog>
            <div class="shadow border border-surface flex-1 auto-rows-fr max-h-full overflow-auto rounded bg-surface-elevated-0">
              <div class="grid grid-cols-[auto_auto_1fr_auto] items-center gap-x-1">
                <div
                  v-for="bp of sim.breakpoints"
                  :key="bp.addr"
                  class="grid grid-cols-subgrid col-span-4 items-center border-t last:border-b border-surface px-4 even:bg-surface-elevated-1"
                >
                  <div>
                    <MdiCircleMedium class="breakpoint-icon icon-active" />
                  </div>
                  <Checkbox
                    v-model="bp.enabled"
                    binary
                    @change="setInternalBreakpointStatus(bp.addr, bp.enabled)"
                  />
                  <div class>
                    <span class="font-mono">
                      {{ toHex(bp.addr) }}
                    </span>
                    <span v-if="bp.addr in memView.symTable">
                      {{ ' \xB7 ' + memView.symTable[bp.addr] }}
                    </span>
                  </div>
                  <button
                    @click="removeBreakpoint(bp.addr)"
                  >
                    <MdiClose
                      width="1em"
                      height="1em"
                    />
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div class="flex flex-col gap-1">
          <div class="header-bar">
            <div />
            <h3 class="header-bar-title">
              Memory
            </h3>
            <OverlayBadge
              severity="primary"
              :value="sim.timer.remaining"
              :class="{ 'hide-badge': !timerRemBadgeShow }"
              :pt="{
                // Offset badge to the left (for timer button)
                pcBadge: {
                  root: {
                    style: {
                      transform: 'translate(-2em, 0%)',
                      'transform-origin': 'right',
                    }
                  }
                }
              }"
            >
              <Button
                v-tooltip.left="'Configure Timer Interrupt'"
                :variant="timerBtnVariant"
                icon="pi"
                rounded
                :severity="timerBtnColor"
                @click="e => timerPopover?.toggle(e)"
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
                <Form
                  v-slot="$form"
                  :initial-values="{
                    vect: 'x' + lc3.getTimerVect().toString(16).padStart(2, '0'),
                    priority: lc3.getTimerPriority(),
                    max: lc3.getTimerMax()
                  }"
                  :validate-on-value-update="false"
                  :resolver="timerValidator"
                  @submit="e => updateTimerProperties(e)"
                >
                  <div>
                    <label>
                      <span>Vector</span>
                      <InputText
                        name="vect"
                        :disabled="!sim.timer.enabled"
                        class="w-24"
                        size="small"
                        :invalid="$form.vect?.invalid"
                      />
                    </label>
                    <Message
                      v-if="$form.vect?.invalid"
                      severity="error"
                      variant="simple"
                      size="small"
                    >
                      {{ $form.vect?.error }}
                    </Message>
                  </div>
                  <label>
                    <span>Priority</span>
                    <InputNumber
                      name="priority"
                      :use-grouping="false"
                      :disabled="!sim.timer.enabled"
                      :min="0"
                      :max="7"
                      input-class="w-24"
                      size="small"
                      :invalid="$form.priority?.invalid"
                    />
                  </label>
                  <label>
                    <span>Repeat</span>
                    <InputNumber
                      name="max"
                      :use-grouping="false"
                      :disabled="!sim.timer.enabled"
                      :min="0"
                      :max="2 ** 31 - 1"
                      input-class="w-24"
                      size="small"
                      :invalid="$form.max?.invalid"
                    />
                  </label>
                  <Button
                    :disabled="!sim.timer.enabled"
                    type="submit"
                    size="small"
                  >
                    Save
                  </Button>
                </Form>
                <Divider />
                <div class="text-center">
                  Interrupt activates in <br>
                  {{ sim.timer.enabled ? sim.timer.remaining : "-" }} instruction{{ sim.timer.remaining !== 1 ? 's' : '' }}
                </div>
              </div>
            </Popover>
          </div>
          <div class="flex grow min-h-0">
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
                  v-for="(item, index) of memView.data"
                  :key="index"
                  :class="{
                    'row-update-flash': item.flash,
                    'row-updated': item.updated,
                    'row-disabled': sim.running,
                    'row-curr-pc': isPCAt(item.addr),
                    'even:bg-surface-elevated-0': memView.data[0]?.addr % 2 == 0,
                    'odd:bg-surface-elevated-0': memView.data[0]?.addr % 2 != 0,
                    scrolling,
                  }"
                  @contextmenu="openMemContextMenu(item)"
                >
                  <td class="data-cell-btn">
                    <div class="flex items-center">
                      <button @click="toggleBreakpoint(item.addr)">
                        <MdiAlertOctagon 
                          class="breakpoint-icon"
                          :class="{
                            'icon-active': isBreakpointAt(item.addr),
                            'text-red-500/50!': sim.breakpoints.some(bp => bp.addr == item.addr && !bp.enabled)
                          }"
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
                    @click="e => showEditPopover('hex', e, item)"
                  >
                    <span>{{ toHex(item.value) }}</span>
                  </td>
                  <td
                    class="data-cell-num clickable"
                    @click="e => showEditPopover('dec', e, item)"
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
          </div>
          <div class="flex items-end pt-5 justify-between">
            <div>
              <Form @submit="e => jumpToMemViewStr(e.states.input.value)">
                <FloatLabel variant="on">
                  <InputText
                    id="jump-loc-input"
                    name="input"
                  />
                  <label for="jump-loc-input">Jump to Location</label>
                </FloatLabel>
              </Form>
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
  

<style scoped>
@reference "@/style.css";

.sim-top:not(.reduce-flashing) .row-disabled {
  @apply text-muted-color bg-emphasis;
  /* For slow computers, add a delay so we aren't flashing the disabled BG repeatedly */
  @apply delay-100;

}
.sim-top.reduce-flashing .row-disabled {
  @apply text-surface-500;
}

.sim-data-table {
  @apply shadow border border-surface table-fixed w-full;
}
.sim-data-table th, .sim-data-table td {
  /* Add padding to all cells */
  /* Hide overlong labels */
  @apply px-2 overflow-hidden whitespace-nowrap;
}
.sim-data-table thead tr {
  @apply bg-table-header;
}
.sim-data-table tr {
  @apply border-b border-surface;
}
.sim-data-table tbody tr:hover {
  @apply bg-emphasis;
}
/* If scrolling, then ignore all color transitions (makes things easier to follow) */
.sim-data-table tbody tr:not(.scrolling) {
  @apply transition-colors;
  .breakpoint-icon, .pc-icon {
    @apply transition;
  }
}

.sim-data-table tbody .data-cell-text, .sim-data-table tbody .data-cell-num {
  @apply font-mono;
}
.data-cell-text {
  @apply text-left;
}
.data-cell-btn {
  @apply text-center;
  button {
    @apply h-6 w-6 flex justify-center items-center;
  }
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
.breakpoint-icon:not(.icon-active), .pc-icon:not(.icon-active) {
  @apply text-surface-400 scale-75;
}
.breakpoint-icon.icon-active {
  @apply text-red-500;
}
.pc-icon.icon-active {
  @apply text-blue-500;
}

tr:not(.row-disabled) .breakpoint-icon:hover {
  @apply text-red-500;
}

tr:not(.row-disabled) .pc-icon:hover {
  @apply text-blue-500;
}

.popover-menu > * {
  @apply flex flex-col gap-3;
}
.popover-menu label {
  @apply flex justify-between items-center gap-2;
}
</style>
