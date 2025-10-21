<!-- eslint-disable vue/no-v-html -->
<!-- 
  SAFETY:
  We are using v-html here, which is unsafe because of XSS concerns.
  However, all user input is escaped into plaintext before being processed,
  and the only possible HTML that can be applied is color (from ansi-to-html),
  so this usage should be fine.
-->

<template>
  <div
    ref="consoleRef"
    class="console"
    :class="{
      'show-focus': props.showFocus,
      'show-cursor': props.showCursor
    }"
    tabindex="0"
    v-html="consoleHtml"
  />
</template>

<script setup lang="ts">
import Convert from 'ansi-to-html';
import { computed, nextTick, ref, useTemplateRef, watch } from 'vue';
const props = defineProps<{
    float?: "top" | "bottom",
    showFocus?: boolean,
    showCursor?: boolean
}>();

/** Maximum number of characters to be displayed in console. */
const DISPLAY_LIMIT = 65536;
const consoleStr = ref("");
const consoleHtml = computed(() => format(consoleStr.value));

const convert = new Convert({
  colors: [
    "#000000", "#CD3131", "#0DBC79", "#E5E510", 
    "#2472C8", "#BC3FBC", "#11A8CD", "#E5E5E5", 
    "#666666", "#F14C4C", "#23D18B", "#F5F543", 
    "#3B8EEA", "#D670D6", "#29B8DB", "#E5E5E5"
  ]
});
/**
 * Format the text for HTML display (dealing with HTML escapes and ANSI escape code colorization).
 */
function format(buf: string): string {
  const string = buf.replace(/[&<>"']/g, m => ({
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#x27;",
  })[m]);

  // Colorize:
  return convert.toHtml(string);
}

/** Returns the internal text buffer. */
function getText(): string {
  return consoleStr.value;
}
/** Appends text to the end of the text buffer. */
function pushText(s: string) {
  if (s.length > 0) {
    setText(consoleStr.value + s);
  }
}
/** Sets the text buffer. */
function setText(s: string) {
  consoleStr.value = removeBackspaces(s).slice(-DISPLAY_LIMIT);
}
defineExpose({ getText, pushText, setText });

/**
 * Remove the backspaces from the given text.
 * If any backspaces (\b) are in the input, then it removes the previous character from the text
 * (as long as the previous character is not a new line).
 */
function removeBackspaces(s: string): string {
  let buf = "";
  let maxErasable = 0; // the maximum number of erasable characters

  // eslint-disable-next-line no-control-regex
  for (const seg of s.split(/(\x08+|\n+)/)) {
    if (seg.startsWith('\b')) {
      if (maxErasable) {
        buf = buf.slice(0, -Math.min(maxErasable, seg.length));
      }
    } else {
      buf += seg;
      // New line => clear "maxErasable"
      maxErasable = seg.startsWith("\n") ? 0 : maxErasable + seg.length;
    }
  }

  return buf;
}

// Handle where we're at:
const consoleRef = useTemplateRef<HTMLDivElement>("consoleRef");
watch(consoleHtml, async () => {
  if (props.float === "top") {
    consoleRef.value.scrollTop = 0;
  } else if (props.float === "bottom") {
    // After DOM updates, scroll to bottom
    await nextTick();
    consoleRef.value.scrollTop = consoleRef.value.scrollHeight;
  } else {
    // Do nothing
  }
});
</script>

<style scoped>
@reference "@/style.css";

.console {
  @apply grow overflow-y-auto;
  @apply font-mono p-2 bg-surface-elevated-0;
  @apply shadow border border-surface rounded;
  white-space: pre-wrap;
  user-select: text;
  overflow-wrap: anywhere;
}

.console.show-focus:focus {
  @apply ring-2 ring-blue-500/50;
}

.console.show-cursor::after {
  content: "\25af";
}
.console.show-cursor:focus::after {
  content: "\25ae";
}
</style>