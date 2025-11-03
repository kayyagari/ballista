<script setup lang="ts">
import { LandingScreenServerStatus } from "~/enums"
import type { Connection } from "~/types"
import { fetch } from "@tauri-apps/plugin-http"

const props = defineProps<{
  server: Connection
}>()

const status = ref<LandingScreenServerStatus>(LandingScreenServerStatus.PENDING)

const emit = defineEmits(["rowClicked", "settingsClicked"])

const doConnectivityCheck = async () => {
  status.value = LandingScreenServerStatus.PENDING

  const address = props.server.address

  // Test basic connectivity
  try {
    await fetch(`${address}/api/system/info`, {
      method: "GET",
      danger: {
        acceptInvalidCerts: true,
        acceptInvalidHostnames: true,
      },
      connectTimeout: 2000,
      headers: {
        "X-Requested-With": "Ballista",
      },
    })
    status.value = LandingScreenServerStatus.AVAILABLE
  } catch (error: any) {
    status.value = LandingScreenServerStatus.UNAVAILABLE
  }
}

await doConnectivityCheck()
</script>

<template>
  <div
    class="hover:bg-[#dcdcdc] hover:cursor-pointer rounded-lg px-2 py-1 flex flex-row items-center justify-between group"
  >
    <section
      class="grow-1 flex flex-row items-center gap-3"
      @click="emit('rowClicked')"
    >
      <p
        class="flex flex-col justify-center items-center font-bold text-white size-10 rounded-full bg-teal-600"
      >
        {{ server.name.replace(/\s/g, "").substring(0, 2) }}
      </p>

      <div>
        <p class="font-bold">{{ server.name }}</p>
        <p>
          <icon
            v-if="status === LandingScreenServerStatus.AVAILABLE"
            name="ph:plugs-connected-bold"
            class="text-lg align-middle text-green-600"
          />
          <icon
            v-else-if="status === LandingScreenServerStatus.UNAVAILABLE"
            name="ph:plugs-bold"
            class="text-lg align-middle text-red-600"
          />
          <icon
            v-else-if="status === LandingScreenServerStatus.PENDING"
            name="ph:circle-notch-bold"
            class="text-lg align-middle text-gray-600 animate-spin"
          />
          {{ server.address }}
        </p>
      </div>
    </section>

    <div
      @click="emit('settingsClicked')"
      class="hidden group-hover:flex flex-col justify-center items-center rounded-full size-8 hover:shadow-lg hover:bg-indigo-900 hover:text-white"
    >
      <icon name="ph:wrench" class="text-xl" />
    </div>
  </div>
</template>
