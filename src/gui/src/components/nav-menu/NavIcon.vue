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
    class="flex items-center justify-center px-3 py-2 hover:bg-surface-500/25 transition"
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

<style lang="postcss" scoped>
.toggled {
  @apply bg-surface-200 dark:bg-surface-700;
}
.toggled :deep(svg) {
    @apply text-sky-600 dark:text-sky-400;
}
:deep(svg) {
    @apply text-surface-600 dark:text-surface-400;
}
.p-overlaybadge :deep(.p-badge) {
  @apply transition;
}
.p-overlaybadge.hide-badge :deep(.p-badge) {
  @apply opacity-0;
}
</style>