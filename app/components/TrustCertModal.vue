<script setup lang="ts">
import type { UntrustedCert } from "~/types"

defineProps<{
  certificate: UntrustedCert
}>()

const emit = defineEmits(["confirm", "cancel"])
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition duration-150 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition duration-100 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        class="fixed inset-0 z-[100] flex items-center justify-center"
      >
        <div
          class="absolute inset-0 bg-black/30 backdrop-blur-sm"
          @click="emit('cancel')"
        />

        <div class="relative bg-surface-1 rounded-xl shadow-overlay w-full max-w-xl mx-4 p-6 space-y-5">
          <div class="flex items-start gap-3">
            <div class="flex-none flex items-center justify-center size-10 rounded-full bg-amber-900/30">
              <icon name="ph:shield-warning" class="text-xl text-amber-400" />
            </div>
            <div>
              <h2 class="font-semibold text-text-primary">Untrusted Certificate</h2>
              <p class="text-sm text-text-tertiary mt-0.5">Do you want to download and run software from this server?</p>
            </div>
          </div>

          <div class="space-y-3 text-sm">
            <div>
              <p class="text-xs font-medium text-text-tertiary uppercase tracking-wider">Subject</p>
              <p class="text-text-primary mt-0.5">{{ certificate.subject }}</p>
            </div>

            <div>
              <p class="text-xs font-medium text-text-tertiary uppercase tracking-wider">Issued By</p>
              <p class="text-text-primary mt-0.5">{{ certificate.issuer }}</p>
            </div>

            <div>
              <p class="text-xs font-medium text-text-tertiary uppercase tracking-wider">Expires On</p>
              <p class="text-text-primary mt-0.5">{{ certificate.expires_on }}</p>
            </div>

            <div>
              <p class="text-xs font-medium text-text-tertiary uppercase tracking-wider">SHA-256 Fingerprint</p>
              <p class="mt-1 font-mono text-xs bg-surface-2 rounded-md px-3 py-2 text-text-secondary break-all leading-relaxed">
                {{ certificate.sha256sum }}
              </p>
            </div>
          </div>

          <div class="flex items-center justify-end gap-2 pt-1">
            <button
              @click="emit('cancel')"
              class="px-3 py-1.5 text-sm rounded-md text-text-secondary hover:bg-surface-2 hover:cursor-pointer transition-colors duration-100"
            >
              Reject
            </button>
            <button
              @click="emit('confirm')"
              class="px-3 py-1.5 text-sm rounded-md bg-accent text-white hover:bg-accent-hover hover:cursor-pointer transition-colors duration-100"
            >
              Trust Certificate
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
