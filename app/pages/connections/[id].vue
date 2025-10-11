<script setup lang="ts">
import type {Connection} from '~/types'
import {invoke} from '@tauri-apps/api/core'
import { ask } from '@tauri-apps/plugin-dialog'

const route = useRoute()
const connectionId = route.params.id

const groups: string[] = await invoke<string[]>('get_all_groups')

const isConnectionEdited = ref<boolean>(false)
const showExtraOptionsDropdown = ref<boolean>(false)

const serverObject: Connection = connectionId === 'new-connection'
    ? await invoke<Connection>('get_default_connectionentry')
    : await invoke<Connection>('load_single_connection', {connectionId: connectionId});

const server = ref<Connection>(serverObject)

watch(
    server,
    () => isConnectionEdited.value = true,
    { deep: true } // Needed to watch any change to the object
)

const handleSave = async () => {
  await invoke('save', { ce: JSON.stringify(server.value) })
  // TODO Error handling
  navigateTo('/')
}

const handleDelete = async () => {
  const confirmed = await ask(`Do you want to delete connection ${server.value.name}?`, { title: 'Are you sure?', kind: 'warning' });
  if (!confirmed) return

  const response = await invoke('delete', { id: server.value.id })

  console.log(response)
  // TODO Error handling
  navigateTo('/')
}
</script>

<template>
  <div class="shrink-0 grow-1 bg-[#faf9fa] px-10 py-5 flex flex-col justify-start gap-10">
    <h1 class="font-bold text-xl">Connection details</h1>

    <form class="grid grid-cols-3 gap-y-3 py-2 pr-5 align-middle items-start overflow-y-scroll">
      <connection-input type="text" label="Name:" v-model="server.name" />

      <connection-input type="text" label="Address:" v-model="server.address" />

      <connection-input type="text" label="Authentication:" v-model="server.username" />

      <connection-input type="password" v-model="server.password" />

      <connection-input type="text" label="Java configuration:" v-model="server.javaHome" />

      <textarea
          class="col-start-2 col-span-2 border-2 border-teal-500 rounded-lg px-2 py-1
            self-start overflow-y-auto resize-y min-h-16 max-w-full"
          placeholder="Additional JVM options"
          v-model="server.javaArgs"
      ></textarea>

      <div
          class="col-span-2 space-x-2 hover:cursor-pointer select-none"
          @click="showExtraOptionsDropdown = !showExtraOptionsDropdown"
      >
        <span class="align-middle">Show more options</span>

        <icon :name="showExtraOptionsDropdown ? 'ph:folder-open-bold' : 'ph:folder-bold'" class="align-middle text-xl" />
      </div>

      <template v-if="showExtraOptionsDropdown">
        <label class="col-start-1 col-span-1 select-none font-bold">Group:</label>

        <insertable-dropdown class="border-2 border-teal-500 rounded-lg col-start-2 col-span-2" :options="groups" v-model="server.group" />

        <connection-input type="text" label="Notes:" v-model="server.notes" />

        <label class="select-none font-bold">Options:</label>
        <div class="col-start-2 col-span-2">
          <input type="checkbox" class="col-span-2" v-model="server.donotcache" />
          Do not cache
        </div>

        <div class="col-start-2 col-span-2">
          <input type="checkbox" class="col-span-2" v-model="server.verify" />
          Verify JAR files
        </div>

      </template>


    </form>

    <div class="grid grid-cols-2 gap-x-3 justify-between">
      <button
          :disabled="!isConnectionEdited"
          @click="handleSave"
          class="
          text-center
          bg-indigo-900
          text-white
          rounded-lg border-2 w-full py-1
          hover:shadow-lg hover:cursor-pointer
          disabled:bg-transparent disabled:border-2 border-stone-400 disabled:text-stone-400 disabled:cursor-not-allowed disabled:shadow-none"
      >
        Save
      </button>
      <button
          @click="handleDelete"
          class="
          text-center
          border-red-500
          text-red-500
          rounded-lg border-2 w-full py-1
          hover:shadow-lg hover:cursor-pointer"
      >
        Delete
      </button>
    </div>
  </div>
</template>