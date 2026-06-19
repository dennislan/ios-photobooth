import { defineStore } from "pinia"
import { invoke } from "@tauri-apps/api/core"

export interface UpdateInfo {
  available: boolean
  currentVersion: string
  latestVersion?: string
  changelog?: string
  mandatory?: boolean
  downloadUrl?: string
  status: "idle" | "checking" | "up-to-date" | "available" | "downloading" | "downloaded" | "error"
  progress: number
  error?: string
}

export const useUpdateStore = defineStore("update", {
  state: (): UpdateInfo => ({
    available: false,
    currentVersion: "1.0.0",
    latestVersion: undefined,
    changelog: undefined,
    mandatory: false,
    downloadUrl: undefined,
    status: "idle",
    progress: 0,
    error: undefined,
  }),

  actions: {
    async checkForUpdates() {
      this.status = "checking"
      this.error = undefined

      try {
        // Get the update endpoint from the Tauri config
        // Falls back to env var PHOTOBOOTH_UPDATE_ENDPOINT
        const endpoint =
          import.meta.env.PHOTOBOOTH_UPDATE_ENDPOINT ||
          "https://your-server.example.com/api/updates/{{target}}/{{current_version}}"

        const result = (await invoke("check_for_updates", { endpoint })) as Record<string, unknown>

        // Fetch the current version from Rust
        const version = (await invoke("get_app_version")) as string
        this.currentVersion = version

        if ((result.available as boolean) === true) {
          this.available = true
          this.latestVersion = (result.latest_version as string) || ""
          this.changelog = (result.changelog as string) || ""
          this.mandatory = (result.mandatory as boolean) || false
          this.downloadUrl = (result.download_url as string) || ""
          this.status = "available"
        } else {
          this.available = false
          this.status = "up-to-date"
        }
      } catch (err) {
        this.status = "error"
        this.error = err instanceof Error ? err.message : String(err)
        // Silently fail — network issues shouldn't crash the app
        console.warn("[update] Check for updates failed:", this.error)
      }
    },

    async applyUpdate() {
      if (!this.available || !this.latestVersion) return

      this.status = "downloading"
      this.progress = 0

      try {
        // Get the app bundle path (defaults to /Applications/photobooth.app)
        // This is passed from the frontend; the Rust side resolves the actual path
        const bundlePath =
          import.meta.env.PHOTOBOOTH_APP_BUNDLE_PATH ||
          "/Applications/photobooth.app"

        const endpoint =
          import.meta.env.PHOTOBOOTH_UPDATE_ENDPOINT ||
          "https://your-server.example.com/api/updates/{{target}}/{{current_version}}"

        await invoke("apply_update", {
          endpoint,
          appBundlePath: bundlePath,
        })

        // After apply_update, the app will quit and restart
        // This code won't execute because the process exits
        this.status = "downloaded"
      } catch (err) {
        this.status = "error"
        this.error = err instanceof Error ? err.message : String(err)
        console.error("[update] Apply update failed:", this.error)
      }
    },

    reset() {
      this.available = false
      this.status = "idle"
      this.progress = 0
      this.error = undefined
      this.latestVersion = undefined
      this.changelog = undefined
    },
  },
})
