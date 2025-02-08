<template>
  <div class="flex flex-col nav-menu">
    <slot />
  </div>
</template>

<style scoped lang="postcss">
.nav-menu {
  @apply bg-surface-50 dark:bg-surface-800 border-r dark:border-surface-700;
}
</style>
