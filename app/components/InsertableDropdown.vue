<script setup lang="ts">
const model = defineModel()

defineProps<{
  options: string[]
}>()

const isOpen = ref<boolean>(false)
const customGroup = ref<string>("")

const handleDropdown = (group: string) => {
  if (group.trim().length === 0) return
  isOpen.value = false
  customGroup.value = ""
  model.value = group
}

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === "Enter") {
    e.preventDefault()
    handleDropdown(customGroup.value)
  }
}
</script>

<template>
  <div class="relative">
    <button
      type="button"
      class="w-full flex items-center justify-between bg-surface-1 border border-border rounded-md px-2.5 py-1.5 text-sm text-text-primary outline-none transition-colors duration-100 hover:cursor-pointer"
      :class="isOpen ? 'border-border-focus ring-1 ring-accent/30' : ''"
      @click="isOpen = !isOpen"
    >
      <span>{{ model }}</span>
      <icon
        name="ph:caret-down"
        class="text-sm text-text-tertiary transition-transform duration-150"
        :class="isOpen ? 'rotate-180' : ''"
      />
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
        v-if="isOpen"
        class="absolute z-10 mt-1 w-full bg-surface-1 border border-border rounded-md shadow-md overflow-hidden"
      >
        <div class="flex items-center gap-1 p-1.5 border-b border-border">
          <input
            type="text"
            placeholder="New group"
            class="flex-1 bg-surface-0 border border-border rounded px-2 py-1 text-sm text-text-primary outline-none focus:border-border-focus focus:ring-1 focus:ring-accent/30"
            v-model="customGroup"
            @keydown="handleKeydown"
          />
          <button
            type="button"
            class="flex items-center justify-center size-7 rounded hover:bg-surface-2 text-text-tertiary hover:text-accent transition-colors duration-100 hover:cursor-pointer"
            @click="handleDropdown(customGroup)"
          >
            <icon name="ph:plus-bold" class="text-sm" />
          </button>
        </div>
        <div class="max-h-32 overflow-y-auto">
          <div
            v-for="(group, groupIndex) in options"
            :key="groupIndex"
            class="px-2.5 py-1.5 text-sm hover:bg-surface-2 hover:cursor-pointer transition-colors duration-75"
            :class="group === model ? 'text-accent font-medium' : 'text-text-primary'"
            @click="handleDropdown(group)"
          >
            {{ group }}
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>
