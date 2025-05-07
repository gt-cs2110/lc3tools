<script setup lang="ts">
    const props = defineProps<{
        label: string,
        badge?: boolean,
        toggle?: boolean
    }>();

    defineEmits(["click"]);
</script>

<template>
  <button
    v-tooltip.right="props.label"
    :aria-label="props.label"
    class="flex items-center justify-center px-3 py-2"
    :class="{'toggled': props.toggle }"
    @click="$emit('click')"
  >
    <OverlayBadge
      value="!"
      severity="warn"
      size="small"
      :class="{ 'hide-badge': !props.badge }"
    >
      <slot />
    </OverlayBadge>
  </button>
</template>

<style scoped>
@reference "@/style.css";

button:hover {
  @apply bg-emphasis;
}
:deep(svg) {
  @apply text-muted-color;
}
button.toggled {
  @apply bg-highlight hover:bg-highlight-emphasis;
}
button.toggled :deep(svg) {
  @apply text-primary;
}
</style>