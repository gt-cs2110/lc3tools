<script setup lang="ts">
    const props = defineProps<{
        label: string,
        badge?: boolean
    }>();

    defineEmits(["click"]);
</script>

<template>
  <button
    v-tooltip.right="props.label"
    :aria-label="props.label"
    class="flex items-center justify-center px-3 py-2 hover:bg-surface-500/25 transition"
    @click="$emit('click')"
  >
    <OverlayBadge
      value="!"
      severity="warn"
      :class="{ 'hide-badge': !props.badge }"
    >
      <slot />
    </OverlayBadge>
  </button>
</template>

<style lang="postcss" scoped>
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