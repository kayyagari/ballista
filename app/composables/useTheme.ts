// Copyright (c) Diridium Technologies Inc. All rights reserved.
// Licensed under the MPL-2.0 License. See LICENSE file in the project root.

import { ref } from "vue"

type Theme = "dark" | "light"

const STORAGE_KEY = "ballista-theme"
const DEFAULT_THEME: Theme = "dark"

const theme = ref<Theme>(DEFAULT_THEME)

function applyTheme(t: Theme) {
  theme.value = t
  document.documentElement.setAttribute("data-theme", t)
  localStorage.setItem(STORAGE_KEY, t)
}

export function useTheme() {
  function init() {
    const saved = localStorage.getItem(STORAGE_KEY) as Theme | null
    applyTheme(saved === "light" ? "light" : DEFAULT_THEME)
  }

  function toggle() {
    applyTheme(theme.value === "dark" ? "light" : "dark")
  }

  return { theme, toggle, init }
}
