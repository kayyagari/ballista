<script setup lang="ts">
import type {Connection, UntrustedCert} from '~/types'
import {invoke} from '@tauri-apps/api/core'

const isLoading = ref<boolean>(false)
const searchFilter = ref<string>('')
const cert = ref<UntrustedCert>()
const connectionToLaunch = ref<Connection>()

const servers: Connection[] = JSON.parse(await invoke('load_connections'))
const filteredServers = computed(() => servers.filter((server) => {
  const search = searchFilter.value.toLowerCase()

  if (!search.length) return true

  const name = server.name.toLowerCase()
  const url = server.address.toLowerCase()
  return name.includes(search) || url.includes(search)
}))

const mappedServers = computed(() => Object.groupBy(filteredServers.value, ({group}) => group))

const handleLaunchClick = async (connection: Connection) => {
  connectionToLaunch.value = connection;
  await launchServer()
}

const launchServer = async () => {
  isLoading.value = true

  // FIXME This is a nasty hack to get the loading icon
  setTimeout(async () => {
    const response: string = await invoke('launch', {id: connectionToLaunch.value!!.id})
    const result: any = JSON.parse(response)

    if (result.code === 1) {
      isLoading.value = false
      cert.value = result.cert
    }

    isLoading.value = false
  }, 100)
}

const trustCertAndLaunch = async () => {
  const result = await invoke("trust_cert", { cert: cert.value.der });
  cert.value = undefined
  console.log(result)
  await launchServer()
}

const openSettings = (server: Connection) => navigateTo(`/connections/${server.id}`)
</script>

<template>
  <div class="bg-[#faf9fa] px-10 py-5 flex flex-col justify-start gap-2 select-none">
    <Teleport to="body">
      <div
          v-if="isLoading"
          class="absolute top-0 left-0 h-screen w-screen z-100 flex justify-center items-center"
      >
        <span class="absolute h-full w-full opacity-25 bg-neutral-900"></span>
        <!--icon name="ph:circle-notch-bold" class="text-7xl text-teal-500 animate-spin" /-->
        <p class="relative opacity-100 bg-white rounded-lg px-5 py-4 text-5xl border-3 border-teal-500">Connecting...</p>
      </div>
    </Teleport>
    <Teleport to="body">
      <div
          v-if="cert"
          class="absolute top-0 left-0 h-screen w-screen z-100 flex justify-center items-center"
      >
        <span class="absolute h-full w-full opacity-25 bg-neutral-900"></span>
        <trust-certificate-dialog
            :cert="cert"
            class="relative opacity-100"
            @noClicked="cert = undefined"
            @yesClicked="trustCertAndLaunch"
        />
      </div>
    </Teleport>

    <input
        type="text"
        placeholder="Search servers by name or IP..."
        v-model="searchFilter"
        class="bg-[#f7f9fa] border-1 border-border rounded-lg py-1 px-2 w-full"
    />

    <div v-for="[group, servers] of Object.entries(mappedServers || {})" :key="group">
      <h1 class="space-x-2">
        <icon name="ph:folder-open-bold" class="align-middle text-xl" />
        <span class="font-bold align-middle">{{group}}</span>
      </h1>

      <ol class="ml-5">
        <li v-for="(server, serverIndex) in servers" :key="serverIndex">
          <brief-server-info @rowClicked="handleLaunchClick(server)" @settingsClicked="openSettings(server)" :server="server" />
        </li>
      </ol>
    </div>

    <button class="
      absolute bottom-12 right-12
      rounded-lg border-2 size-11
      hover:shadow-2xl hover:border-teal-500 hover:text-teal-500 hover:cursor-pointer
    "
    @click="navigateTo('/connections/new-connection')">
      <icon name="ph:plus-bold" class="align-middle text-xl" />
    </button>
  </div>
</template>
