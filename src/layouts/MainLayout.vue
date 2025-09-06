<template>
  <q-layout view="lHh Lpr lFf">
    <q-page-container>
      <router-view />
    </q-page-container>
  </q-layout>
  
</template>

<script setup lang="ts">
// Minimal layout: no header, no drawer, no toolbar
</script>

<style scoped>
/* Ensure the layout/page uses full viewport for the game canvas */
:deep(.q-page-container) {
  padding: 0;
}
</style>
