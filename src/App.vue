<template>
  <div class="h-screen flex flex-col bg-bg-primary">
    <!-- 更新提示横幅 -->
    <UpdateBanner />

    <!-- 顶部导航栏 -->
    <header
      class="flex items-center justify-between px-6 py-3 bg-white border-b border-border-color shadow-sm-custom z-100"
    >
      <div class="flex items-center gap-3">
        <svg
          class="w-7 h-7 text-primary"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
          <circle cx="8.5" cy="8.5" r="1.5" />
          <polyline points="21 15 16 10 5 21" />
        </svg>
        <h1 class="text-xl font-bold text-text-primary m-0">大头贴</h1>
      </div>

      <a-tabs v-model:active-key="currentView" size="small">
        <a-tab-pane key="idle" tab="待机" />
        <a-tab-pane key="capture" tab="拍照" />
        <a-tab-pane key="selection" tab="选片" />
        <a-tab-pane key="preview" tab="打印" />
      </a-tabs>

      <div>
        <a-button
          type="text"
          icon-role="icon"
          @click="showSettings = true"
          class="!p-2 !rounded-full hover:bg-bg-tertiary"
        >
          <template #icon>
            <SettingOutlined />
          </template>
        </a-button>
      </div>
    </header>

    <!-- 主内容区 -->
    <main class="flex-1 overflow-hidden relative" role="main">
      <IdleView v-if="currentView === 'idle'" @start-capture="goToCapture" />
      <CaptureView
        v-if="currentView === 'capture'"
        @complete="currentView = 'selection'"
      />
      <SelectionView
        v-if="currentView === 'selection'"
        @confirm="currentView = 'preview'"
        @back="currentView = 'capture'"
      />
      <PreviewView
        v-if="currentView === 'preview'"
        @back="currentView = 'selection'"
      />
    </main>

    <!-- 设置面板 -->
    <Teleport to="body">
      <SettingsPanel :visible="showSettings" @close="showSettings = false" />
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue"
import { SettingOutlined } from "@ant-design/icons-vue"
import IdleView from "./components/IdleView.vue"
import CaptureView from "./views/CaptureView.vue"
import SelectionView from "./views/SelectionView.vue"
import PreviewView from "./views/PreviewView.vue"
import SettingsPanel from "./components/SettingsPanel.vue"
import UpdateBanner from "./components/UpdateBanner.vue"
import { useUpdateStore } from "./stores/update"

const currentView = ref<"idle" | "capture" | "selection" | "preview">("idle")
const showSettings = ref(false)
const updateStore = useUpdateStore()

const goToCapture = () => {
  currentView.value = "capture"
}

onMounted(() => {
  // Handle deep link on mount
  const url = window.location.href
  if (url.startsWith("photobooth://")) {
    handleDeepLink(url)
  }

  // Check for updates in background (silent, non-blocking)
  updateStore.checkForUpdates()
})

function handleDeepLink(url: string) {
  if (url.startsWith("photobooth://")) {
    console.log("Deep link received:", url)
    currentView.value = "capture"
  }
}
</script>
