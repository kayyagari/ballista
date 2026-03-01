<script setup lang="ts">
import { LandingScreenServerStatus } from "~/enums"
import type { Connection } from "~/types"

const avatarColors = [
  "bg-blue-600",
  "bg-emerald-600",
  "bg-violet-600",
  "bg-amber-600",
  "bg-rose-600",
  "bg-cyan-600",
]

const props = defineProps<{
  server: Connection
  selected: boolean
  status?: LandingScreenServerStatus
}>()

const emit = defineEmits(["select", "launch", "edit"])

const avatarColor = computed(() => {
  let hash = 0
  for (let i = 0; i < props.server.name.length; i++) {
    hash = props.server.name.charCodeAt(i) + ((hash << 5) - hash)
  }
  return avatarColors[Math.abs(hash) % avatarColors.length]
})

const formatRelativeTime = (timestamp: number | null): string => {
  if (!timestamp) return ""
  const diff = Date.now() - timestamp
  const seconds = Math.floor(diff / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)

  if (seconds < 60) return "just now"
  if (minutes < 60) return `${minutes}m ago`
  if (hours < 24) return `${hours}h ago`
  if (days < 7) return `${days}d ago`
  if (days < 30) return `${Math.floor(days / 7)}w ago`
  return `${Math.floor(days / 30)}mo ago`
}

const lastConnectedLabel = computed(() => formatRelativeTime(props.server.lastConnected))

const currentStatus = computed(() => props.status ?? LandingScreenServerStatus.PENDING)
const isAvailable = computed(() => currentStatus.value === LandingScreenServerStatus.AVAILABLE)
</script>

<template>
  <div
    class="flex items-center justify-between rounded-md px-2 py-1.5 transition-colors duration-[120ms] hover:cursor-pointer group"
    :class="selected ? 'bg-surface-2' : 'hover:bg-surface-2'"
    @click="emit('select')"
    @dblclick="emit('launch')"
  >
    <section class="flex-1 flex items-center gap-3 min-w-0">
      <div
        class="flex-none flex items-center justify-center font-semibold text-white size-8 rounded-md text-xs"
        :class="avatarColor"
      >
        {{ server.name.replace(/\s/g, "").substring(0, 2) }}
      </div>

      <div class="min-w-0 flex-1">
        <div class="flex items-center justify-between gap-2">
          <p class="font-medium text-sm text-text-primary truncate">{{ server.name }}</p>
          <span v-if="lastConnectedLabel" class="flex-none text-xs text-text-tertiary">{{ lastConnectedLabel }}</span>
        </div>
        <p class="text-xs text-text-secondary flex items-center gap-1.5">
          <span
            class="inline-block size-1.5 rounded-full flex-none"
            :class="{
              'bg-status-available': currentStatus === LandingScreenServerStatus.AVAILABLE,
              'bg-status-unavailable': currentStatus === LandingScreenServerStatus.UNAVAILABLE,
              'bg-status-pending animate-pulse': currentStatus === LandingScreenServerStatus.PENDING,
            }"
          />
          <span class="truncate">{{ server.address }}</span>
          <template v-if="server.username">
            <span class="text-text-disabled">·</span>
            <span class="flex-none text-text-tertiary">{{ server.username }}</span>
          </template>
        </p>
      </div>
    </section>

    <div class="flex-none flex items-center gap-1 ml-2">
      <button
        @click.stop="emit('launch')"
        class="flex items-center justify-center size-7 rounded-md transition-all duration-100"
        :class="isAvailable
          ? 'hover:bg-surface-3 text-accent hover:text-accent-hover hover:cursor-pointer'
          : 'text-text-disabled cursor-default'"
      >
        <icon name="ph:play-fill" class="text-sm" />
      </button>
      <button
        @click.stop="emit('edit')"
        class="flex items-center justify-center size-7 rounded-md hover:bg-surface-3 text-text-tertiary hover:text-text-primary transition-all duration-100 hover:cursor-pointer"
      >
        <icon name="ph:pencil-simple" class="text-sm" />
      </button>
    </div>
  </div>
</template>
