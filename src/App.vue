<template>
  <div id="app-container">
    <!-- 顶部导航栏 -->
    <header class="app-header">
      <div class="header-left">
        <svg
          class="logo-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
          <circle cx="8.5" cy="8.5" r="1.5" />
          <polyline points="21 15 16 10 5 21" />
        </svg>
        <h1 class="logo-text">android大头贴</h1>
      </div>

      <nav class="header-nav" role="navigation" aria-label="主导航">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="nav-btn"
          :class="{ active: currentView === tab.id }"
          @click="currentView = tab.id"
          :aria-label="tab.label"
          :aria-current="currentView === tab.id ? 'page' : undefined"
        >
          <svg
            class="nav-icon"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <component :is="tab.icon" />
          </svg>
          <span class="nav-label">{{ tab.label }}</span>
        </button>
      </nav>

      <div class="header-right">
        <button
          class="btn-icon settings-btn"
          @click="showSettings = true"
          aria-label="打开设置"
        >
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <circle cx="12" cy="12" r="3" />
            <path
              d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"
            />
          </svg>
        </button>
      </div>
    </header>

    <!-- 主内容区 -->
    <main class="app-main" role="main">
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
      <SettingsPanel v-if="showSettings" @close="showSettings = false" />
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue"
import IdleView from "./components/IdleView.vue"
import CaptureView from "./views/CaptureView.vue"
import SelectionView from "./views/SelectionView.vue"
import PreviewView from "./views/PreviewView.vue"
import SettingsPanel from "./components/SettingsPanel.vue"

// Icon components
const CameraIcon = {
  template:
    '<circle cx="12" cy="12" r="3"/><path d="M23 11a2 2 0 0 1-2 2h-1v1a3 3 0 0 1-3 3H8a3 3 0 0 1-3-3v-1H4a2 2 0 0 1-2-2z"/>',
}

const ImageIcon = {
  template:
    '<rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/>',
}

const PrintIcon = {
  template:
    '<polyline points="6 9 6 2 18 2 18 9"/><path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"/><rect x="6" y="14" width="12" height="8"/>',
}

const tabs = [
  { id: "idle" as const, label: "待机", icon: CameraIcon },
  { id: "capture" as const, label: "拍照", icon: CameraIcon },
  { id: "selection" as const, label: "选片", icon: ImageIcon },
  { id: "preview" as const, label: "打印", icon: PrintIcon },
]

const currentView = ref<"idle" | "capture" | "selection" | "preview">("idle")
const showSettings = ref(false)

const goToCapture = () => {
  currentView.value = "capture"
}

onMounted(() => {
  // Handle deep link on mount
  const url = window.location.href
  if (url.startsWith("android-photo://")) {
    handleDeepLink(url)
  }
})

function handleDeepLink(url: string) {
  if (url.startsWith("android-photo://")) {
    console.log("Deep link received:", url)
    currentView.value = "capture"
  }
}
</script>

<style scoped>
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 24px;
  background: white;
  border-bottom: 1px solid var(--border-color);
  box-shadow: var(--shadow-sm);
  z-index: 100;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo-icon {
  width: 28px;
  height: 28px;
  color: var(--android-blue);
}

.logo-text {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
}

.header-nav {
  display: flex;
  gap: 8px;
}

.nav-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border: 1px solid var(--border-color);
  background: transparent;
  color: var(--text-secondary);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-base);
  font-size: 14px;
  font-weight: 500;
  min-height: 40px;
}

.nav-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.nav-btn.active {
  background: var(--android-blue);
  color: white;
  border-color: var(--android-blue);
}

.nav-icon {
  width: 16px;
  height: 16px;
}

.header-right {
  display: flex;
  align-items: center;
}

.settings-btn {
  padding: 8px;
  border-radius: var(--radius-full);
}

.app-main {
  flex: 1;
  overflow: hidden;
  position: relative;
}

@media (max-width: 768px) {
  .app-header {
    padding: 8px 16px;
  }

  .nav-label {
    display: none;
  }

  .nav-btn {
    padding: 8px;
  }
}
</style>
