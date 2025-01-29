<template>
  <div class="flex flex-col nav-menu">
    <slot />
  </div>
</template>

<style scoped lang="postcss">
.nav-menu {
  @apply bg-white dark:bg-zinc-800 border-r dark:border-zinc-700;
}
</style>
