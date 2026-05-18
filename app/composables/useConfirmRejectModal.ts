// Copyright (c) Diridium Technologies Inc. All rights reserved.
// Licensed under the MPL-2.0 License. See LICENSE file in the project root.

import { createVNode, getCurrentInstance, render } from "vue"
import TrustCertModal from "~/components/TrustCertModal.vue"
import type { UntrustedCert } from "~/types"

export function useConfirmRejectModal() {
  const appContext = getCurrentInstance()?.appContext

  const mountModal = (component: any, props: Record<string, any>) => {
    return new Promise((resolve) => {
      const container = document.createElement("div")
      document.body.appendChild(container)

      const vnode = createVNode(component, {
        ...props,
        onConfirm: () => {
          resolve(true)
          cleanup()
        },
        onCancel: () => {
          resolve(false)
          cleanup()
        },
      })

      if (appContext) {
        vnode.appContext = appContext
      }

      render(vnode, container)

      const cleanup = () => {
        render(null, container)
        document.body.removeChild(container)
      }
    })
  }

  return {
    trustCertificate: (certificate: UntrustedCert) => {
      return mountModal(TrustCertModal, { certificate })
    },
  }
}
