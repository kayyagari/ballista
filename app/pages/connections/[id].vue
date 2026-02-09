<script setup lang="ts">
import type { Connection } from "~/types"
import { invoke } from "@tauri-apps/api/core"
import { ask } from "@tauri-apps/plugin-dialog"

const route = useRoute()
const connectionId = route.params.id

const isNewConnection = connectionId === "new-connection"

const groups: string[] = await invoke<string[]>("get_all_groups")

const isConnectionEdited = ref<boolean>(false)

const serverObject: Connection =
  isNewConnection
    ? await invoke<Connection>("get_default_connectionentry")
    : await invoke<Connection>("load_single_connection", {
        connectionId: connectionId,
      })

const server = ref<Connection>(serverObject)

watch(
  server,
  () => (isConnectionEdited.value = true),
  { deep: true },
)

const handleSave = async () => {
  await invoke("save", { ce: JSON.stringify(server.value) })
  navigateTo("/")
}

const handleCancel = async () => {
  if (isConnectionEdited.value) {
    const confirmed = await ask(
      "You have unsaved changes. Discard them?",
      { title: "Discard changes?", kind: "warning" },
    )
    if (!confirmed) return
  }
  navigateTo("/")
}

const handleDelete = async () => {
  const confirmed = await ask(
    `Do you want to delete connection ${server.value.name}?`,
    { title: "Are you sure?", kind: "warning" },
  )
  if (!confirmed) return

  await invoke("delete", { id: server.value.id })
  navigateTo("/")
}
</script>

<template>
  <div class="bg-surface-0 flex flex-col h-full overflow-hidden">
    <!-- Header -->
    <div class="px-5 pt-5 pb-4">
      <h1 class="font-semibold text-lg text-text-primary">
        {{ isNewConnection ? "New Connection" : "Edit Connection" }}
      </h1>
    </div>

    <!-- Scrollable form area -->
    <div class="flex-1 overflow-y-auto px-5 pb-24">
      <form class="grid grid-cols-2 gap-x-8 gap-y-6" @submit.prevent>
        <!-- Left column: Connection -->
        <section class="space-y-3">
          <h2 class="text-xs font-medium text-text-tertiary uppercase tracking-wider">Connection</h2>
          <connection-input type="text" label="Name" placeholder="My Server" v-model="server.name" />
          <connection-input type="text" label="Address" placeholder="https://hostname:8443" v-model="server.address" />
        </section>

        <!-- Right column: Java -->
        <section class="space-y-3">
          <h2 class="text-xs font-medium text-text-tertiary uppercase tracking-wider">Configuration</h2>
          <connection-input type="text" label="Java Home" placeholder="/usr/lib/jvm/java-11" v-model="server.javaHome" />
          <div class="space-y-1">
            <label class="block text-sm font-medium text-text-secondary select-none">JVM Arguments</label>
            <textarea
              class="w-full bg-surface-1 border border-border rounded-md px-2.5 py-1.5 text-sm text-text-primary placeholder:text-text-disabled outline-none transition-colors duration-100 focus:border-border-focus focus:ring-1 focus:ring-accent/30 resize-y min-h-16"
              placeholder="Additional JVM options"
              v-model="server.javaArgs"
            ></textarea>
          </div>
        </section>

        <!-- Left column: Authentication -->
        <section class="space-y-3">
          <h2 class="text-xs font-medium text-text-tertiary uppercase tracking-wider">Authentication</h2>
          <connection-input type="text" label="Username" placeholder="admin" v-model="server.username" />
          <connection-input type="password" label="Password" v-model="server.password" />
        </section>

        <!-- Right column: Group, Notes, Options -->
        <section class="space-y-3">
          <h2 class="text-xs font-medium text-text-tertiary uppercase tracking-wider">Organization</h2>
          <div class="space-y-1">
            <label class="block text-sm font-medium text-text-secondary select-none">Group</label>
            <insertable-dropdown :options="groups" v-model="server.group" />
          </div>
          <connection-input type="text" label="Heap Size" placeholder="512m" v-model="server.heapSize" />
          <connection-input type="text" label="Notes" placeholder="Optional notes" v-model="server.notes" />
          <div class="space-y-2 pt-1">
            <p class="text-sm font-medium text-text-secondary select-none">Options</p>
            <label class="flex items-center gap-2 text-sm text-text-primary hover:cursor-pointer select-none">
              <input type="checkbox" class="accent-accent" v-model="server.showConsole" />
              Show Java console
            </label>
            <label class="flex items-center gap-2 text-sm text-text-primary hover:cursor-pointer select-none">
              <input type="checkbox" class="accent-accent" v-model="server.donotcache" />
              Do not cache
            </label>
            <label class="flex items-center gap-2 text-sm text-text-primary hover:cursor-pointer select-none">
              <input type="checkbox" class="accent-accent" v-model="server.verify" />
              Verify JAR files
            </label>
          </div>
        </section>
      </form>
    </div>

    <!-- Action bar -->
    <div class="flex-none flex items-center justify-between px-5 py-3 border-t border-border bg-surface-0">
      <button
        @click="handleCancel"
        class="px-3 py-1.5 text-sm rounded-md text-text-secondary hover:bg-surface-2 hover:cursor-pointer transition-colors duration-100"
      >
        Cancel
      </button>
      <div class="flex items-center gap-2">
        <button
          v-if="!isNewConnection"
          @click="handleDelete"
          class="px-3 py-1.5 text-sm rounded-md text-danger hover:bg-danger/10 hover:cursor-pointer transition-colors duration-100"
        >
          Delete
        </button>
        <button
          :disabled="!isConnectionEdited"
          @click="handleSave"
          class="px-4 py-1.5 text-sm rounded-md bg-accent text-white hover:bg-accent-hover hover:cursor-pointer transition-colors duration-100 disabled:opacity-40 disabled:cursor-not-allowed"
        >
          {{ isNewConnection ? "Create" : "Save" }}
        </button>
      </div>
    </div>
  </div>
</template>
