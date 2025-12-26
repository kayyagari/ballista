<script setup lang="ts">
const model = defineModel()

defineProps<{
  label?: string
  type: string
}>()

const inputId = useId()
</script>

<template>
  <label :for="inputId" v-if="label" class="select-none font-bold">{{ label }}</label>
  <input
    :id="inputId"
    :type="type"
    class="col-start-2 col-span-2 border-2 border-teal-500 rounded-lg px-2 py-1"
    v-model="model"
  />
</template>
