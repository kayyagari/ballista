import tailwindcss from "@tailwindcss/vite"

export default defineNuxtConfig({
  compatibilityDate: "2025-05-15",

  devtools: { enabled: true },

  ssr: false,

  devServer: {
    host: "0",
  },

  app: {
    head: {
      script: [
        {
          innerHTML: `(function(){var t=localStorage.getItem('launcher-theme');if(t==='light')document.documentElement.setAttribute('data-theme','light')})()`,
          type: "text/javascript",
        },
      ],
    },
    pageTransition: {
      name: "slide", // we'll define CSS for "slide"
      mode: "out-in", // waits for leave before enter
    },
  },

  modules: ["@nuxt/icon"],

  css: ["~/assets/css/main.css"],

  vite: {
    plugins: [tailwindcss()],

    clearScreen: false,
    envPrefix: ["VITE_", "TAURI_"],
    server: {
      strictPort: true,
    },
  },
  ignore: ["**/src-tauri/**"],
})
