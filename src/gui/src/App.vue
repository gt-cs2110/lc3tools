<template>
    <h1>💖 Hello World!</h1>
    <p>Welcome to your Electron application.</p>

    <RouterLink to="/editor">ausstieg links</RouterLink><br>
    <RouterLink to="/simulator">ausstieg rechts</RouterLink>

    <div class="bordered">
      <RouterView />
    </div>
  </template>
  
<script setup lang="ts">
  import { RouterLink } from "vue-router";
  import * as renderer from "./types/renderer";
  declare var lc3: renderer.LC3Backend;

  console.log('👋 This message is being logged by "App.vue", included via Vite');
  console.log(lc3.addInput);
</script>

<style lang="css">
  @import "index.css";

  .bordered {
    border: 1px solid black;
  }
</style>