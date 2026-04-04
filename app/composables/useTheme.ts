// Copyright (c) Diridium Technologies Inc. All rights reserved.
// Licensed under the MPL-2.0 License. See LICENSE file in the project root.

import { ref } from "vue"

type Theme = "dark" | "light"

const STORAGE_KEY = "ballista-theme"
const DEFAULT_THEME: Theme = "dark"

const theme = ref<Theme>(DEFAULT_THEME)
let systemThemeListenerRegistered = false

function getSystemThemeMediaQuery() {
  if (typeof window === "undefined") {
    return null
  }

  return window.matchMedia("(prefers-color-scheme: dark)")
}

function getSystemTheme(): Theme {
  const mediaQuery = getSystemThemeMediaQuery()
  if (!mediaQuery) {
    return DEFAULT_THEME
  }

  return mediaQuery.matches ? "dark" : "light"
}

function applyTheme(t: Theme) {
  theme.value = t
  document.documentElement.setAttribute("data-theme", t)
  document.documentElement.style.colorScheme = t
  localStorage.setItem(STORAGE_KEY, t)
}

export function useTheme() {
  function init() {
    const saved = localStorage.getItem(STORAGE_KEY) as Theme | null
    applyTheme(saved === "light" ? "light" : getSystemTheme());

    if (systemThemeListenerRegistered) {
      return
    }

    const mediaQuery = getSystemThemeMediaQuery()
    if (!mediaQuery) {
      return
    }

    mediaQuery.addEventListener("change", (event) => {
      applyTheme(event.matches ? "dark" : "light")
    })
    systemThemeListenerRegistered = true
  }

  function toggle() {
    applyTheme(theme.value === "dark" ? "light" : "dark")
  }

  return { theme, toggle, init }
}
