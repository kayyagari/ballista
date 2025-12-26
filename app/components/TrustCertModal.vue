<script setup lang="ts">
import type { UntrustedCert } from "~/types"

defineProps<{
  certificate: UntrustedCert
}>()

const emit = defineEmits(["confirm", "cancel"])
</script>

<template>
  <Teleport to="body">
    <div
      class="absolute top-0 left-0 h-screen w-screen z-100 flex justify-center items-center"
    >
      <!-- Tinted background  -->
      <span class="absolute h-full w-full opacity-25 bg-neutral-900"></span>

      <!-- Dialog itself -->
      <div
        class="container bg-white rounded-2xl p-12 space-y-6 relative opacity-100"
      >
        <p>
          <b>Untrusted Certificate</b>
        </p>

        <p>
          <b>Subject:</b>
          {{ certificate.subject }}
        </p>

        <p>
          <b>Issued By:</b>
          {{ certificate.issuer }}
        </p>

        <p>
          <b>Expires on:</b>
          {{ certificate.expires_on }}
        </p>

        <p>Do you trust this certificate?</p>

        <div class="text-lg space-x-3 text-right">
          <button
            @click="emit('cancel')"
            class="hover:cursor-pointer hover:shadow-xl w-16 px-2 py-1 rounded-lg border-1"
          >
            No
          </button>
          <button
            @click="emit('confirm')"
            class="hover:cursor-pointer hover:shadow-xl w-16 px-2 py-1 rounded-lg border-1 border-teal-500 bg-teal-500 text-white"
          >
            Yes
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
