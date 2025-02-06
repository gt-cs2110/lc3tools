<script setup lang="ts">
    const props = defineProps<{
        label: string,
        badge?: boolean
    }>();

    defineEmits(["click"]);
</script>

<template>
  <Button
    v-tooltip.right="props.label"
    variant="text"
    :aria-label="props.label"
    class="nav-icon-btn"
    @click="$emit('click')"
  >
    <OverlayBadge
      value="!"
      severity="warn"
      :class="{ 'hide-badge': !props.badge }"
    >
      <slot
        name="default"
        class="nav-icon"
      />
    </OverlayBadge>
  </Button>
</template>

<style lang="postcss" scoped>
.nav-icon-btn :deep(svg) {
    @apply text-surface-600 dark:text-surface-400;
}
.p-overlaybadge :deep(.p-badge) {
  @apply transition;
}
.p-overlaybadge.hide-badge :deep(.p-badge) {
  @apply opacity-0;
}
</style>