/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly PHOTOBOOTH_UPDATE_ENDPOINT?: string
  readonly PHOTOBOOTH_APP_BUNDLE_PATH?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
