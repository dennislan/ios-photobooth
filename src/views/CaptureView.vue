<template>
  <div class="capture-view">
    <!-- 投屏区域 -->
    <div class="mirror-container">
      <div v-if="!isScrcpyRunning" class="mirror-placeholder">
        <div class="placeholder-icon">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
          >
            <rect x="5" y="2" width="14" height="20" rx="2" ry="2" />
            <line x1="12" y1="18" x2="12.01" y2="18" />
          </svg>
        </div>
        <p class="placeholder-title">连接 android 手机</p>
        <p class="placeholder-desc">请启用 USB 调试模式</p>
        <div class="device-status" v-if="deviceId">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            class="status-icon"
          >
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
            <polyline points="22 4 12 14.01 9 11.01" />
          </svg>
          设备: {{ deviceId }}
        </div>
        <button class="btn btn-primary connect-btn" @click="startScrcpy">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            class="btn-icon-svg"
          >
            <path d="M5 12h14M12 5l7 7-7 7" />
          </svg>
          连接手机
        </button>
      </div>
      <video
        v-else
        ref="videoEl"
        class="mirror-video"
        autoplay
        muted
        playsinline
        aria-label="手机投屏画面"
      />
      <div class="scrcpy-badge" v-if="isScrcpyRunning">
        <span class="status-dot active"></span>
        投屏中
      </div>
    </div>

    <!-- 模式选择 -->
    <div class="mode-selector" role="radiogroup" aria-label="拍照模式">
      <button
        v-for="mode in captureModes"
        :key="mode.id"
        class="mode-btn"
        :class="{ active: captureStore.mode === mode.id }"
        @click="captureStore.setMode(mode.id)"
        role="radio"
        :aria-checked="captureStore.mode === mode.id"
      >
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          class="mode-icon"
        >
          <component :is="mode.icon" />
        </svg>
        <span>{{ mode.label }}</span>
      </button>
    </div>

    <!-- 拍照按钮 -->
    <div class="capture-controls">
      <button
        class="btn btn-success take-photo-btn"
        :disabled="
          !isScrcpyRunning ||
          captureStore.photos.length >= captureStore.maxPhotos
        "
        @click="takePhoto"
        aria-label="拍照，已拍 {{ captureStore.photos.length }} 张，共 {{ captureStore.maxPhotos }} 张"
      >
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          class="btn-icon-svg"
        >
          <path
            d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"
          />
          <circle cx="12" cy="13" r="4" />
        </svg>
        拍照 ({{ captureStore.photos.length }}/{{ captureStore.maxPhotos }})
      </button>
    </div>

    <!-- 照片预览 -->
    <div
      class="photo-strip"
      v-if="captureStore.photos.length > 0"
      role="list"
      aria-label="已拍摄照片"
    >
      <div
        v-for="(photo, idx) in captureStore.photos"
        :key="idx"
        class="photo-thumb"
        :class="{ selected: captureStore.selectedIndex === idx }"
        @click="captureStore.selectPhoto(idx)"
        role="listitem"
        tabindex="0"
        :aria-label="`照片 ${idx + 1}${photo.isLive ? ', Live Photo' : ''}`"
      >
        <img :src="photo.dataUrl" alt="" loading="lazy" />
        <button
          class="remove-btn"
          @click.stop="removePhoto(idx)"
          aria-label="删除照片"
        >
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
        <span class="photo-badge" v-if="photo.isLive">LIVE</span>
      </div>
    </div>

    <!-- 完成按钮 -->
    <button
      v-if="captureStore.photos.length > 0"
      class="btn btn-primary complete-btn"
      @click="$emit('complete')"
    >
      下一步
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        class="btn-icon-svg"
      >
        <polyline points="9 18 15 12 9 6" />
      </svg>
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted } from "vue"
import { useCaptureStore } from "../stores/capture"
import { useScrcpyStore } from "../stores/scrcpy"

defineEmits<{
  complete: []
}>()

const captureStore = useCaptureStore()
const scrcpyStore = useScrcpyStore()
const videoEl = ref<HTMLVideoElement | null>(null)
const deviceId = ref("")
let ws: WebSocket | null = null

const isScrcpyRunning = computed(() => scrcpyStore.isConnected)

const captureModes = [
  { id: "grid4" as const, label: "四宫格", icon: "grid" },
  { id: "newspaper" as const, label: "报纸机", icon: "newspaper" },
  { id: "live" as const, label: "Live Photo", icon: "live" },
]

async function startScrcpy() {
  try {
    const { invoke } = await import("@tauri-apps/api/core")
    const devices: string[] = await invoke("get_device_list")
    if (devices.length === 0) {
      alert("未检测到设备，请检查 USB 连接和调试模式")
      return
    }
    deviceId.value = devices[0]
    await invoke("start_scrcpy", { deviceId: devices[0] })
    scrcpyStore.setState({
      connected: true,
      running: true,
      deviceId: devices[0],
    })
    connectWebSocket()
  } catch (e) {
    alert(e instanceof Error ? e.message : String(e))
  }
}

function connectWebSocket() {
  ws = new WebSocket("ws://localhost:27183")
  ws.onopen = () => console.log("WebSocket connected")
  ws.onmessage = (event) => {
    if (event.data instanceof Blob) {
      // Handle video frames
    }
  }
  ws.onerror = (error) => console.error("WebSocket error:", error)
}

async function takePhoto() {
  try {
    const { invoke } = await import("@tauri-apps/api/core")
    const devId = scrcpyStore.state.deviceId
    if (!devId) {
      alert("请先连接设备")
      return
    }
    const photoPath: string = await invoke("take_photo", { deviceId: devId })
    const thumbnail: string = await invoke("get_photo_thumbnail", {
      filename: photoPath,
      deviceId: devId,
      flip: false,
    })
    const isLive: boolean = await invoke("check_live_photo", {
      filename: photoPath,
    })
    captureStore.addPhoto({
      dataUrl: `data:image/jpeg;base64,${thumbnail}`,
      filePath: photoPath,
      isLive,
    })
  } catch (e) {
    alert(e instanceof Error ? e.message : String(e))
  }
}

function removePhoto(index: number) {
  captureStore.removePhoto(index)
}

onUnmounted(() => {
  if (ws) ws.close()
})
</script>

<style scoped>
.capture-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: var(--space-md);
  padding: var(--space-md);
}

.mirror-container {
  flex: 1;
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
  border: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  position: relative;
}

.mirror-placeholder {
  text-align: center;
  color: var(--text-secondary);
  padding: var(--space-xl);
}

.placeholder-icon {
  width: 64px;
  height: 64px;
  margin: 0 auto var(--space-lg);
  color: var(--text-tertiary);
}

.placeholder-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: var(--space-sm);
}

.placeholder-desc {
  font-size: 14px;
  margin-bottom: var(--space-lg);
}

.device-status {
  display: inline-flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-md);
  background: rgba(0, 122, 255, 0.1);
  border-radius: var(--radius-full);
  color: var(--android-blue);
  font-size: 14px;
  font-weight: 500;
  margin-bottom: var(--space-lg);
}

.status-icon {
  width: 16px;
  height: 16px;
}

.connect-btn {
  margin-top: var(--space-md);
}

.mirror-video {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.scrcpy-badge {
  position: absolute;
  top: var(--space-md);
  right: var(--space-md);
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: rgba(0, 0, 0, 0.7);
  border-radius: var(--radius-full);
  color: white;
  font-size: 12px;
  font-weight: 500;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--android-red);
}

.status-dot.active {
  background: var(--android-green);
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.mode-selector {
  display: flex;
  gap: var(--space-sm);
}

.mode-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px;
  border: 1px solid var(--border-color);
  background: transparent;
  color: var(--text-secondary);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-base);
  font-size: 14px;
  font-weight: 500;
  min-height: 44px;
}

.mode-btn:hover {
  background: var(--bg-tertiary);
}

.mode-btn.active {
  background: var(--android-blue);
  color: white;
  border-color: var(--android-blue);
}

.mode-icon {
  width: 16px;
  height: 16px;
}

.capture-controls {
  text-align: center;
}

.take-photo-btn {
  padding: 12px 32px;
  font-size: 16px;
  min-height: 48px;
}

.btn-icon-svg {
  width: 20px;
  height: 20px;
}

.photo-strip {
  display: flex;
  gap: var(--space-sm);
  overflow-x: auto;
  padding: var(--space-xs) 0;
}

.photo-thumb {
  position: relative;
  width: 80px;
  height: 80px;
  border-radius: var(--radius-md);
  border: 2px solid transparent;
  overflow: hidden;
  cursor: pointer;
  flex-shrink: 0;
  transition: border-color var(--transition-base);
}

.photo-thumb.selected {
  border-color: var(--android-blue);
}

.photo-thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.remove-btn {
  position: absolute;
  top: 4px;
  right: 4px;
  width: 24px;
  height: 24px;
  background: rgba(0, 0, 0, 0.6);
  color: white;
  border: none;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background var(--transition-base);
}

.remove-btn:hover {
  background: rgba(255, 59, 48, 0.8);
}

.remove-btn svg {
  width: 12px;
  height: 12px;
}

.photo-badge {
  position: absolute;
  bottom: 4px;
  left: 4px;
  padding: 2px 6px;
  background: rgba(0, 0, 0, 0.7);
  color: white;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.5px;
}

.complete-btn {
  padding: 12px;
  font-size: 16px;
  min-height: 48px;
}

@media (max-width: 768px) {
  .capture-view {
    padding: var(--space-sm);
    gap: var(--space-sm);
  }

  .photo-thumb {
    width: 64px;
    height: 64px;
  }
}
</style>
