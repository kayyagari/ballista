<script setup lang="ts">
import type { LandingScreenServer } from "~/types"

defineProps<{
  server: LandingScreenServer
}>()
</script>

<template>
  <div
    class="border-border border-2 rounded-lg p-4 hover:shadow-xl hover:cursor-pointer group flex flex-row justify-between items-center"
  >
    <div>
      <div class="font-bold text-xl truncate">
        {{ server.name }}
      </div>

      <div>
        <div class="rounded-full size-2 inline-block" :class="server.status" />
        {{ server.address }}
      </div>
    </div>

    <p class="flex-none hidden group-hover:inline-block">
      <icon
        name="ph:wrench"
        class="text-right text-2xl hover:text-gray-400 hover:shadow-xl"
      />
    </p>
  </div>
</template>
