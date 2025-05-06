<template>
  <div class="flex flex-col nav-menu">
    <slot />
  </div>
</template>

<style scoped>
@reference "@/style.css";

.nav-menu {
  @apply bg-surface-50 dark:bg-surface-800 border-r border-surface;
}
</style>
