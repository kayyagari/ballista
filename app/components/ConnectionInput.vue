<script setup lang="ts">
const model = defineModel()

defineProps<{
  label?: string
  type: string
  placeholder?: string
}>()

const inputId = useId()
</script>

<template>
  <div class="space-y-1">
    <label
      :for="inputId"
      v-if="label"
      class="block text-sm font-medium text-text-secondary select-none"
    >
      {{ label }}
    </label>
    <input
      :id="inputId"
      :type="type"
      :placeholder="placeholder"
      class="w-full bg-surface-1 border border-border rounded-md px-2.5 py-1.5 text-sm text-text-primary placeholder:text-text-disabled outline-none transition-colors duration-100 focus:border-border-focus focus:ring-1 focus:ring-accent/30"
      v-model="model"
    />
  </div>
</template>
