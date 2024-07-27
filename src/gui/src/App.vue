<template>
    <h1>💖 Hello World!</h1>
    <p>Welcome to your Electron application.</p>
  </template>
  
<script setup lang="ts">
  // Declare lc3's presence.
  import type LC3Backend from "lc3-backend";
  declare var lc3: typeof LC3Backend;

  console.log('👋 This message is being logged by "App.vue", included via Vite');
  console.log(lc3.addInput);
</script>

<style lang="css">
  @import "index.css";
</style>