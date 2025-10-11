<script setup lang="ts">
const model = defineModel()

defineProps<{
  label?: string,
  type: string,
}>()
</script>

<template>
  <label v-if="label" class="select-none font-bold">{{ label }}</label>
  <input
      :type="type"
      class="col-start-2 col-span-2 border-2 border-teal-500 rounded-lg px-2 py-1"
      v-model="model"
  />
</template>