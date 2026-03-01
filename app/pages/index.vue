<script setup lang="ts">
import type { Connection } from "~/types"
import { LandingScreenServerStatus } from "~/enums"
import { Channel, invoke } from "@tauri-apps/api/core"
import { fetch as tauriFetch } from "@tauri-apps/plugin-http"
import { ask, open } from "@tauri-apps/plugin-dialog"
import { open as shellOpen } from "@tauri-apps/plugin-shell"

type SortMode = "group" | "name" | "lastConnected" | "status"

const isLoading = ref<boolean>(false)
const progressMessage = ref<string>("Connecting...")
const launchError = ref<string | null>(null)
const searchFilter = ref<string>("")
const selectedServerId = ref<string | null>(null)
const sortBy = ref<SortMode>((localStorage.getItem("launcher-sort") as SortMode) || "group")
const showSortMenu = ref(false)

watch(sortBy, (v) => {
  localStorage.setItem("launcher-sort", v)
  showSortMenu.value = false
})

const servers: Connection[] = JSON.parse(await invoke("load_connections"))

// Connectivity status tracking (lifted from BriefServerInfo)
const serverStatuses = reactive<Record<string, LandingScreenServerStatus>>({})

const checkConnectivity = async (server: Connection) => {
  serverStatuses[server.id] = LandingScreenServerStatus.PENDING
  try {
    await tauriFetch(`${server.address}/api/system/info`, {
      method: "GET",
      danger: { acceptInvalidCerts: true, acceptInvalidHostnames: true },
      connectTimeout: 2000,
      headers: { "X-Requested-With": "Launcher" },
    })
    serverStatuses[server.id] = LandingScreenServerStatus.AVAILABLE
  } catch {
    serverStatuses[server.id] = LandingScreenServerStatus.UNAVAILABLE
  }
}

onMounted(() => servers.forEach(checkConnectivity))

const filteredServers = computed(() =>
  servers.filter((server) => {
    const search = searchFilter.value.toLowerCase()
    if (!search.length) return true
    const name = server.name.toLowerCase()
    const url = server.address.toLowerCase()
    return name.includes(search) || url.includes(search)
  }),
)

const sortedServers = computed(() => {
  const list = [...filteredServers.value]
  switch (sortBy.value) {
    case "name":
      return list.sort((a, b) => a.name.localeCompare(b.name))
    case "lastConnected":
      return list.sort((a, b) => (b.lastConnected ?? 0) - (a.lastConnected ?? 0))
    case "status":
      return list.sort((a, b) => {
        const order = { [LandingScreenServerStatus.AVAILABLE]: 0, [LandingScreenServerStatus.PENDING]: 1, [LandingScreenServerStatus.UNAVAILABLE]: 2 }
        return (order[serverStatuses[a.id] ?? 2] ?? 2) - (order[serverStatuses[b.id] ?? 2] ?? 2)
      })
    default:
      return list
  }
})

const isGrouped = computed(() => sortBy.value === "group")

const mappedServers = computed(() =>
  Object.groupBy(filteredServers.value, ({ group }) => group),
)

const collapsedGroups = reactive<Set<string>>(
  new Set(JSON.parse(localStorage.getItem("launcher-collapsed-groups") || "[]")),
)

const toggleGroup = (group: string) => {
  if (collapsedGroups.has(group)) {
    collapsedGroups.delete(group)
  } else {
    collapsedGroups.add(group)
  }
  localStorage.setItem("launcher-collapsed-groups", JSON.stringify([...collapsedGroups]))
}

const hasServers = computed(() => servers.length > 0)
const hasResults = computed(() => filteredServers.value.length > 0)

const { trustCertificate } = useConfirmRejectModal()
const handleLaunchClick = (connection: Connection) => {
  isLoading.value = true
  launchError.value = null
  progressMessage.value = "Connecting..."
  nextTick(() => launchServer(connection))
}

const launchServer = async (connection: Connection) => {
  const onProgress = new Channel<{ message: string }>()
  onProgress.onmessage = ({ message }) => {
    progressMessage.value = message
  }

  try {
    // Loop to handle multiple untrusted certs across different jars
    while (true) {
      const response: string = await invoke("launch", {
        id: connection.id,
        on_progress: onProgress,
      })
      const result = JSON.parse(response)

      // Result code 1 means cert needs trust approval
      if (result.code !== 1) return

      const shouldTrustCertificate = await trustCertificate(result.cert)
      if (!shouldTrustCertificate) return

      await invoke("trust_cert", { cert: result.cert.der })
    }
  } catch (e) {
    launchError.value = `Launch failed: ${e}`
  } finally {
    isLoading.value = false
  }
}

const openSettings = (server: Connection) =>
  navigateTo(`/connections/${server.id}`)

const importConnections = async () => {
  const filePath = await open({
    filters: [{ name: "JSON", extensions: ["json"] }],
    multiple: false,
  })
  if (!filePath) return
  try {
    const resp: string = await invoke("import", { file_path: filePath, overwrite: false })
    const result = JSON.parse(resp)
    if (result.status === "duplicates") {
      const names = result.names.join(", ")
      const confirmed = await ask(
        `${result.names.length} of ${result.total} connections already exist and will be overwritten:\n\n${names}`,
        { title: "Overwrite existing connections?", kind: "warning" },
      )
      if (!confirmed) return
      await invoke("import", { file_path: filePath, overwrite: true })
    }
    window.location.reload()
  } catch (e) {
    launchError.value = `Import failed: ${e}`
  }
}

const refreshStatuses = () => {
  servers.forEach(checkConnectivity)
}

const showAbout = ref(false)

const openHelp = async () => {
  const confirmed = await ask("This will open the Launcher wiki in your default browser. Continue?", {
    title: "Open Help",
    kind: "info",
  })
  if (confirmed) {
    await shellOpen("https://github.com/pacmano1/launcher/wiki")
  }
}

const deselectAll = () => {
  selectedServerId.value = null
  showSortMenu.value = false
}
</script>

<template>
  <div class="bg-surface-0 flex flex-col h-full select-none overflow-hidden">
    <!-- Header -->
    <div class="flex items-center justify-between px-5 pt-5 pb-3">
      <div class="flex items-center gap-2">
        <h1 class="font-semibold text-lg text-text-primary">Launcher</h1>
        <button
          @click="showAbout = true"
          class="flex items-center justify-center size-6 rounded-md text-text-disabled hover:text-text-tertiary hover:cursor-pointer transition-colors duration-100"
        >
          <icon name="ph:info" class="text-sm" />
        </button>
        <button
          @click="openHelp"
          data-tooltip="Open wiki in browser"
          data-tooltip-below
          class="flex items-center justify-center size-6 rounded-md text-text-disabled hover:text-text-tertiary hover:cursor-pointer transition-colors duration-100"
        >
          <icon name="ph:question" class="text-sm" />
        </button>
      </div>
      <div class="flex items-center gap-2">
        <button
          class="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-md border border-border bg-surface-1 text-text-secondary hover:text-text-primary hover:bg-surface-2 hover:cursor-pointer transition-colors duration-100"
          @click="importConnections"
          data-tooltip="Import connections from JSON file"
        >
          <icon name="ph:download-simple-bold" class="text-xs" />
          Import
        </button>
        <button
          class="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-md bg-accent text-white hover:bg-accent-hover hover:cursor-pointer transition-colors duration-100"
          @click="navigateTo('/connections/new-connection')"
        >
          <icon name="ph:plus-bold" class="text-xs" />
          Add
        </button>
      </div>
    </div>

    <!-- Search + Sort -->
    <div class="flex items-center gap-2 px-5 pb-3">
      <div class="relative flex-1">
        <icon
          name="ph:magnifying-glass"
          class="absolute left-2.5 top-1/2 -translate-y-1/2 text-sm text-text-tertiary"
        />
        <input
          type="text"
          placeholder="Search servers..."
          v-model="searchFilter"
          class="w-full bg-surface-1 border border-border rounded-md py-1.5 pl-8 pr-3 text-sm text-text-primary placeholder:text-text-disabled outline-none transition-colors duration-100 focus:border-border-focus focus:ring-1 focus:ring-accent/30"
        />
      </div>
      <button
        @click="refreshStatuses"
        data-tooltip="Refresh server status"
        class="flex items-center justify-center size-8 rounded-md border border-border bg-surface-1 text-text-tertiary hover:text-text-primary hover:cursor-pointer transition-colors duration-100"
      >
        <icon name="ph:arrow-clockwise" class="text-sm" />
      </button>
      <div class="relative">
        <button
          @click="showSortMenu = !showSortMenu"
          class="flex items-center justify-center size-8 rounded-md border border-border bg-surface-1 text-text-tertiary hover:text-text-primary hover:cursor-pointer transition-colors duration-100"
          :class="showSortMenu ? 'border-border-focus text-text-primary' : ''"
        >
          <icon name="ph:sort-ascending" class="text-sm" />
        </button>
        <Transition
          enter-active-class="transition duration-100 ease-out"
          enter-from-class="opacity-0 scale-95"
          enter-to-class="opacity-100 scale-100"
          leave-active-class="transition duration-75 ease-in"
          leave-from-class="opacity-100 scale-100"
          leave-to-class="opacity-0 scale-95"
        >
          <div
            v-if="showSortMenu"
            class="absolute right-0 top-full mt-1 z-50 w-44 bg-surface-1 border border-border rounded-md shadow-lg py-1"
          >
            <button
              v-for="option in ([
                { value: 'group', label: 'Group', icon: 'ph:folders' },
                { value: 'name', label: 'Name', icon: 'ph:sort-ascending' },
                { value: 'lastConnected', label: 'Last connected', icon: 'ph:clock' },
                { value: 'status', label: 'Status', icon: 'ph:circle-half' },
              ] as const)"
              :key="option.value"
              @click="sortBy = option.value"
              class="flex items-center gap-2 w-full px-3 py-1.5 text-xs hover:bg-surface-2 transition-colors duration-75 hover:cursor-pointer"
              :class="sortBy === option.value ? 'text-accent' : 'text-text-secondary'"
            >
              <icon :name="option.icon" class="text-sm" />
              {{ option.label }}
              <icon v-if="sortBy === option.value" name="ph:check-bold" class="text-xs ml-auto" />
            </button>
          </div>
        </Transition>
      </div>
    </div>

    <!-- Server list -->
    <div class="flex-1 overflow-y-auto px-5 pb-5" @click.self="deselectAll">
      <!-- No servers empty state -->
      <div
        v-if="!hasServers"
        class="flex flex-col items-center justify-center h-full text-center"
      >
        <icon name="ph:hard-drives" class="text-4xl text-text-disabled mb-3" />
        <p class="font-medium text-text-secondary">No servers yet</p>
        <p class="text-sm text-text-tertiary mt-1">Add a connection to get started.</p>
        <button
          class="mt-4 flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-md bg-accent text-white hover:bg-accent-hover hover:cursor-pointer transition-colors duration-100"
          @click="navigateTo('/connections/new-connection')"
        >
          <icon name="ph:plus-bold" class="text-xs" />
          Add Server
        </button>
      </div>

      <!-- No search results empty state -->
      <div
        v-else-if="!hasResults"
        class="flex flex-col items-center justify-center h-full text-center"
      >
        <icon name="ph:magnifying-glass" class="text-4xl text-text-disabled mb-3" />
        <p class="font-medium text-text-secondary">No results</p>
        <p class="text-sm text-text-tertiary mt-1">
          No servers matching "{{ searchFilter }}"
        </p>
      </div>

      <!-- Server groups (grouped mode) -->
      <div v-else-if="isGrouped" class="space-y-4" @click.self="deselectAll">
        <div
          v-for="[group, groupServers] of Object.entries(mappedServers || {})"
          :key="group"
          @click.self="deselectAll"
        >
          <button
            class="flex items-center gap-1.5 text-xs font-medium text-text-tertiary uppercase tracking-wider px-2 mb-1 hover:text-text-secondary transition-colors duration-100 hover:cursor-pointer"
            @click="toggleGroup(group)"
          >
            <icon
              name="ph:caret-right-bold"
              class="text-[10px] transition-transform duration-150"
              :class="collapsedGroups.has(group) ? '' : 'rotate-90'"
            />
            {{ group }}
            <span class="normal-case tracking-normal font-normal">({{ groupServers?.length ?? 0 }})</span>
          </button>

          <div v-if="!collapsedGroups.has(group)" class="space-y-px">
            <brief-server-info
              v-for="server in groupServers"
              :key="server.id"
              :server="server"
              :status="serverStatuses[server.id]"
              :selected="selectedServerId === server.id"
              @select="selectedServerId = server.id"
              @launch="handleLaunchClick(server)"
              @edit="openSettings(server)"
            />
          </div>
        </div>
      </div>

      <!-- Flat sorted list -->
      <div v-else class="space-y-px" @click.self="deselectAll">
        <brief-server-info
          v-for="server in sortedServers"
          :key="server.id"
          :server="server"
          :status="serverStatuses[server.id]"
          :selected="selectedServerId === server.id"
          @select="selectedServerId = server.id"
          @launch="handleLaunchClick(server)"
          @edit="openSettings(server)"
        />
      </div>
    </div>

    <!-- Bottom status bar -->
    <Transition
      enter-active-class="transition duration-150 ease-out"
      enter-from-class="translate-y-full opacity-0"
      enter-to-class="translate-y-0 opacity-100"
      leave-active-class="transition duration-100 ease-in"
      leave-from-class="translate-y-0 opacity-100"
      leave-to-class="translate-y-full opacity-0"
    >
      <div v-if="isLoading" class="flex-none border-t border-border bg-surface-1">
        <div class="h-0.5 bg-surface-2 overflow-hidden">
          <div class="h-full w-1/3 bg-accent rounded-full animate-[statusSlide_1.5s_ease-in-out_infinite]" />
        </div>
        <div class="flex items-center gap-2 px-4 py-2">
          <icon name="ph:circle-notch-bold" class="text-sm text-accent animate-spin flex-none" />
          <p class="text-xs text-text-secondary truncate">{{ progressMessage }}</p>
        </div>
      </div>
    </Transition>

    <!-- Launch error -->
    <Transition
      enter-active-class="transition duration-150 ease-out"
      enter-from-class="translate-y-full opacity-0"
      enter-to-class="translate-y-0 opacity-100"
      leave-active-class="transition duration-100 ease-in"
      leave-from-class="translate-y-0 opacity-100"
      leave-to-class="translate-y-full opacity-0"
    >
      <div v-if="launchError" class="absolute bottom-0 inset-x-0 bg-danger/10 border-t border-danger/30">
        <div class="flex items-center justify-between px-4 py-2">
          <p class="text-xs text-danger truncate">{{ launchError }}</p>
          <button @click="launchError = null" class="text-xs text-danger hover:text-text-primary hover:cursor-pointer ml-2 flex-none">Dismiss</button>
        </div>
      </div>
    </Transition>

    <!-- About modal -->
    <Transition
      enter-active-class="transition duration-150 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition duration-100 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div v-if="showAbout" class="absolute inset-0 z-[100] flex items-center justify-center bg-black/50" @click.self="showAbout = false">
        <div class="bg-surface-1 border border-border rounded-lg shadow-overlay w-80 p-5">
          <div class="flex items-center justify-between mb-4">
            <h2 class="font-semibold text-text-primary">About Launcher</h2>
            <button @click="showAbout = false" class="text-text-tertiary hover:text-text-primary hover:cursor-pointer">
              <icon name="ph:x" class="text-sm" />
            </button>
          </div>
          <div class="space-y-3 text-sm">
            <p class="text-text-secondary">Version 2.0.0</p>
            <div class="space-y-1">
              <p class="text-text-secondary">Originally created by <span class="text-text-primary">Kiran Ayyagari</span></p>
              <p class="text-text-secondary">Modifications by <span class="text-text-primary">Diridium Technologies Inc.</span></p>
            </div>
            <p class="text-text-tertiary text-xs">Licensed under MPL-2.0</p>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>
