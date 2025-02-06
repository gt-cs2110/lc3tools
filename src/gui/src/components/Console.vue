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
import { computed, nextTick, useTemplateRef, watch } from 'vue';
const props = defineProps<{
    float?: "top" | "bottom",
    showFocus?: boolean,
    showCursor?: boolean
}>();

const consoleStr = defineModel<string, string>();

const convert = new Convert({
  colors: [
    "#000000", "#CD3131", "#0DBC79", "#E5E510", 
    "#2472C8", "#BC3FBC", "#11A8CD", "#E5E5E5", 
    "#666666", "#F14C4C", "#23D18B", "#F5F543", 
    "#3B8EEA", "#D670D6", "#29B8DB", "#E5E5E5"
  ]
});
const consoleHtml = computed(() => {
  // Handle backspaces:
  const buf: string[] = [];
  // pattern represents: (ANSI escape code | new line | any non-new-line character)
  // ANSI escape code is of format: \x1B[999;999;999
  // eslint-disable-next-line no-control-regex
  for (const ch of consoleStr.value.match(/(?:\x1B\[(?:\d+;)*\d+m|\n|.)/g) ?? []) {
    if (ch === '\b') {
        if (buf.length === 0) continue;
        if (buf[buf.length - 1] === '\n') continue;
        buf.pop();
    } else {
        buf.push(ch);
    }
  }

  // Escape console string:
  const string = buf.join("").replace(/[&<>"']/g, m => ({
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#x27;",
  })[m]);

  // Colorize:
  return convert.toHtml(string);
});

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

<style scoped lang="postcss">
.console {
  @apply grow overflow-y-auto;
  @apply font-mono p-2 dark:bg-surface-800;
  @apply border shadow dark:border-surface-800 transition;
  white-space: pre-wrap;
  user-select: text;
  overflow-wrap: anywhere;
}

.console.show-focus:focus {
  @apply shadow-blue-500/40;
  box-shadow: 0px 0px 6px 3px var(--tw-shadow-color) !important;
}

.console.show-cursor::after {
  content: "\25af";
}
.console.show-cursor:focus::after {
  content: "\25ae";
}
</style>