<script setup lang="ts"></script>

<template>
  <div class="bg-surface-0 flex flex-col h-full">
    <div class="flex items-center gap-3 px-5 pt-5 pb-3">
      <button
        @click="navigateTo('/')"
        class="flex items-center justify-center size-7 rounded-md hover:bg-surface-2 text-text-tertiary hover:text-text-primary transition-colors duration-100 hover:cursor-pointer"
      >
        <icon name="ph:arrow-left" class="text-base" />
      </button>
      <h1 class="font-semibold text-lg text-text-primary">Settings</h1>
    </div>

    <div class="flex-1 flex flex-col items-center justify-center text-center">
      <icon name="ph:gear" class="text-4xl text-text-disabled mb-3" />
      <p class="font-medium text-text-secondary">Coming Soon</p>
      <p class="text-sm text-text-tertiary mt-1">Settings will be available in a future update.</p>
    </div>
  </div>
</template>
