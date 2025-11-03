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
  model.value = group
}
</script>

<template>
  <div class="relative">
    <div class="flex flex-row items-center px-2 py-1">
      <span class="align-middle grow-1">{{ model }}</span>
      <icon
        :name="
          isOpen ? 'ph:arrow-circle-down-bold' : 'ph:arrow-circle-left-bold'
        "
        class="align-middle right-0 text-2xl hover:cursor-pointer"
        @click="isOpen = !isOpen"
      />
    </div>

    <ol
      v-if="isOpen"
      class="border-2 border-teal-500 rounded-b-lg absolute z-10 bg-gray-100 w-full max-w-full space-y-1 p-1"
    >
      <li class="flex flex-row items-center space-x-2">
        <input
          type="text"
          placeholder="New group"
          class="grow-1"
          v-model="customGroup"
        />
        <icon
          name="ph:plus-circle-bold"
          class="align-middle px-2 py-1 text-2xl hover:cursor-pointer"
          @click="handleDropdown(customGroup)"
        />
      </li>
      <li
        v-for="(group, groupIndex) in options"
        :key="groupIndex"
        class="hover:bg-blue-600 hover:cursor-pointer"
        @click="handleDropdown(group)"
      >
        {{ group }}
      </li>
    </ol>
  </div>
</template>
