import { createVNode, render } from "vue"
import TrustCertModal from "~/components/TrustCertModal.vue"
import type { UntrustedCert } from "~/types"

export function useConfirmRejectModal() {
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
