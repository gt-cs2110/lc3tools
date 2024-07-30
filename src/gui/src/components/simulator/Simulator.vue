<template>
    <v-navigation-drawer permanent rail>
    <v-list-item @click="todo()" prepend-icon="folder_open">
      <v-tooltip location="right" activator="parent" text="Open File" />
    </v-list-item>
    <v-list-item @click="() => running = !running">
      <template v-slot:prepend>
        <v-icon v-if="!running" icon="play_arrow" />
        <v-icon v-else icon="pause" />
      </template>
      <v-tooltip location="right" activator="parent" v-if="!running" text="Run" />
      <v-tooltip location="right" activator="parent" v-else text="Pause" />
    </v-list-item>
    <v-list-item @click="todo()" prepend-icon="refresh">
      <v-tooltip location="right" activator="parent" text="Reload Object Files" />
    </v-list-item>
    <v-list-item @click="todo()" prepend-icon="redo">
      <v-tooltip location="right" activator="parent" text="Step Over" />
    </v-list-item>
    <v-list-item @click="todo()" prepend-icon="subdirectory_arrow_right">
      <v-tooltip location="right" activator="parent" text="Step In" />
    </v-list-item>
    <v-list-item @click="todo()" prepend-icon="subdirectory_arrow_left">
      <v-tooltip location="right" activator="parent" text="Step Out" />
    </v-list-item>
    <v-list-item @click="todo()" prepend-icon="power_settings_new">
      <v-tooltip location="right" activator="parent" text="Reinitialize Machine" />
    </v-list-item>
    <v-list-item @click="todo()" prepend-icon="shuffle">
      <v-tooltip location="right" activator="parent" text="Randomize Machine" />
    </v-list-item>
  </v-navigation-drawer>
  <!-- Main editor content -->
  <v-main>
    <v-container>
      <h1>💖 Hello World!</h1>
      <p>Welcome to your Electron simulator.</p>
    </v-container>
  </v-main>
</template>
  
<script setup lang="ts">
import { ref } from 'vue';

  console.log('Simulator.');
  const running = ref(false);

  function todo() {
    throw new Error("TODO");
  }
</script>