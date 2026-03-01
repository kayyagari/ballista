<script setup lang="ts">
useZoom()
const { init: initTheme } = useTheme()

onMounted(() => {
  initTheme()
  document.addEventListener("contextmenu", (e) => e.preventDefault())
})
</script>

<template>
  <nuxt-page class="h-screen overflow-hidden" />
</template>
