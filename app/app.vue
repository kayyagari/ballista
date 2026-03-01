<script setup lang="ts">
useZoom()

onMounted(() => {
  document.addEventListener("contextmenu", (e) => e.preventDefault())
})
</script>

<template>
  <nuxt-page class="h-screen overflow-hidden" />
</template>
