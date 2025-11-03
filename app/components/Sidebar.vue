<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core"
import type { BallistaInfo } from "~/types"

const ballistaInfoResult: string = await invoke("get_ballista_info")
const ballistaInfo: BallistaInfo = JSON.parse(ballistaInfoResult)

const routes: { title: string; to: string }[] = [
  { title: "Servers", to: "/" },
  { title: "Settings", to: "/settings" },
]
</script>

<template>
  <div
    class="bg-linear-to-tr from-indigo-900 to-teal-500 text-white flex flex-col justify-around items-center px-10"
  >
    <p class="text-center">
      <span class="font-bold text-2xl block">Ballista</span>
      <span class="block">Version: {{ ballistaInfo.ballista_version }}</span>
    </p>

    <div class="flex flex-col gap-4 w-full">
      <nuxt-link
        v-for="(route, routeIndex) in routes"
        :key="routeIndex"
        :to="route.to"
        class="rounded-lg border-2 w-full py-1 hover:shadow-lg hover:cursor-pointer text-center"
      >
        {{ route.title }}
      </nuxt-link>
    </div>
  </div>
</template>
