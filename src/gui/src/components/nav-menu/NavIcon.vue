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
      v-if="props.badge"
      value="!"
      severity="warn"
    >
      <slot
        name="default"
        class="nav-icon"
      />
    </OverlayBadge>
    <slot
      v-else
      name="default"
      class="nav-icon"
    />
  </Button>
</template>

<style lang="postcss" scoped>
.nav-icon-btn * {
    @apply text-stone-600 dark:text-stone-400;
}
</style>