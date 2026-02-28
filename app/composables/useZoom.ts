import { ref, onMounted, onUnmounted } from "vue"
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow"

const ZOOM_STEP = 0.1
const ZOOM_MIN = 0.5
const ZOOM_MAX = 2.0
const DEFAULT_ZOOM = 1.0
const STORAGE_KEY = "ballista-zoom-level"

const zoomLevel = ref(DEFAULT_ZOOM)

export function useZoom() {
  async function setZoom(level: number) {
    const clamped = Math.round(Math.max(ZOOM_MIN, Math.min(ZOOM_MAX, level)) * 10) / 10
    zoomLevel.value = clamped
    localStorage.setItem(STORAGE_KEY, String(clamped))
    const webview = getCurrentWebviewWindow()
    await webview.setZoom(clamped)
  }

  function zoomIn() {
    setZoom(zoomLevel.value + ZOOM_STEP)
  }

  function zoomOut() {
    setZoom(zoomLevel.value - ZOOM_STEP)
  }

  function zoomReset() {
    setZoom(DEFAULT_ZOOM)
  }

  function handleKeydown(e: KeyboardEvent) {
    const mod = e.metaKey || e.ctrlKey
    if (!mod) return

    if (e.key === "=" || e.key === "+") {
      e.preventDefault()
      zoomIn()
    } else if (e.key === "-") {
      e.preventDefault()
      zoomOut()
    } else if (e.key === "0") {
      e.preventDefault()
      zoomReset()
    }
  }

  onMounted(() => {
    const saved = localStorage.getItem(STORAGE_KEY)
    if (saved) setZoom(parseFloat(saved))
    window.addEventListener("keydown", handleKeydown)
  })

  onUnmounted(() => {
    window.removeEventListener("keydown", handleKeydown)
  })

  return { zoomLevel, zoomIn, zoomOut, zoomReset }
}
