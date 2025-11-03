<script setup lang="ts">
import type { UntrustedCert } from "~/types"

defineProps<{
  cert: UntrustedCert
}>()

const emit = defineEmits(["noClicked", "yesClicked"])
</script>

<template>
  <div class="container bg-white rounded-2xl p-12 space-y-6">
    <p>
      <b>Untrusted Certificate</b>
    </p>

    <p>
      <b>Subject:</b>
      {{ cert.subject }}
    </p>

    <p>
      <b>Issued By:</b>
      {{ cert.issuer }}
    </p>

    <p>
      <b>Expires on:</b>
      {{ cert.expires_on }}
    </p>

    <p>Do you trust this certificate?</p>

    <div class="text-lg space-x-3 text-right">
      <button
        @click="emit('noClicked')"
        class="hover:cursor-pointer hover:shadow-xl w-16 px-2 py-1 rounded-lg border-1"
      >
        No
      </button>
      <button
        @click="emit('yesClicked')"
        class="hover:cursor-pointer hover:shadow-xl w-16 px-2 py-1 rounded-lg border-1 border-teal-500 bg-teal-500 text-white"
      >
        Yes
      </button>
    </div>
  </div>
</template>
