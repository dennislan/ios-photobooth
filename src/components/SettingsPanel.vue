<template>
  <div
    class="settings-overlay"
    @click.self="$emit('close')"
    role="dialog"
    aria-modal="true"
    aria-label="设置"
  >
    <div class="settings-panel">
      <div class="settings-header">
        <h2>设置</h2>
        <button class="close-btn" @click="$emit('close')" aria-label="关闭设置">
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
      </div>

      <div class="settings-content">
        <div class="setting-group">
          <h3>设备设置</h3>
          <div class="setting-item">
            <label>USB 调试</label>
            <span
              class="status"
              :class="{ active: scrcpyStore.state.connected }"
            >
              {{ scrcpyStore.state.connected ? "已连接" : "未连接" }}
            </span>
          </div>
          <div class="setting-item">
            <label>设备 ID</label>
            <span class="value">{{ scrcpyStore.state.deviceId || "无" }}</span>
          </div>
        </div>

        <div class="setting-group">
          <h3>拍摄设置</h3>
          <div class="setting-item">
            <label for="maxPhotos">最大照片数</label>
            <input
              id="maxPhotos"
              type="number"
              v-model.number="maxPhotos"
              min="1"
              max="10"
              class="input"
            />
          </div>
          <div class="setting-item">
            <label for="autoFlip">自动翻转</label>
            <input
              id="autoFlip"
              type="checkbox"
              v-model="autoFlip"
              class="checkbox"
            />
          </div>
        </div>

        <div class="setting-group">
          <h3>打印设置</h3>
          <div class="setting-item">
            <label for="paperSize">纸张大小</label>
            <select id="paperSize" v-model="paperSize" class="select">
              <option value="4x6">4x6 英寸</option>
              <option value="5x7">5x7 英寸</option>
              <option value="6x8">6x8 英寸</option>
            </select>
          </div>
          <div class="setting-item">
            <label for="printMode">打印模式</label>
            <select id="printMode" v-model="printMode" class="select">
              <option value="color">彩色</option>
              <option value="bw">黑白</option>
            </select>
          </div>
        </div>

        <div class="setting-group">
          <h3>关于</h3>
          <div class="setting-item">
            <label>版本</label>
            <span class="value">v1.0.0</span>
          </div>
        </div>
      </div>

      <div class="settings-actions">
        <button class="apply-btn" @click="applySettings">应用设置</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue"
import { useScrcpyStore } from "../stores/scrcpy"

defineEmits<{
  close: []
}>()

const scrcpyStore = useScrcpyStore()
const maxPhotos = ref(4)
const autoFlip = ref(false)
const paperSize = ref("4x6")
const printMode = ref("color")

function applySettings() {
  localStorage.setItem(
    "photobooth-settings",
    JSON.stringify({
      maxPhotos: maxPhotos.value,
      autoFlip: autoFlip.value,
      paperSize: paperSize.value,
      printMode: printMode.value,
    }),
  )
  alert("设置已保存")
}
</script>

<style scoped>
.settings-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.settings-panel {
  background: white;
  border-radius: var(--radius-lg);
  width: 90%;
  max-width: 500px;
  max-height: 80vh;
  overflow-y: auto;
  box-shadow: var(--shadow-xl);
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
}

.settings-header h2 {
  margin: 0;
  font-size: 18px;
}

.close-btn {
  background: transparent;
  border: none;
  cursor: pointer;
  color: var(--text-secondary);
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  background: var(--bg-tertiary);
}

.close-btn svg {
  width: 20px;
  height: 20px;
}

.settings-content {
  padding: 20px;
}

.setting-group {
  margin-bottom: 24px;
}

.setting-group h3 {
  margin: 0 0 12px 0;
  font-size: 16px;
  color: var(--android-blue);
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 0;
  border-bottom: 1px solid var(--border-color);
}

.setting-item:last-child {
  border-bottom: none;
}

.setting-item label {
  font-size: 14px;
  color: var(--text-secondary);
  margin: 0;
}

.setting-item .status {
  padding: 4px 10px;
  border-radius: var(--radius-full);
  font-size: 13px;
  font-weight: 500;
}

.setting-item .status.active {
  background: rgba(52, 199, 89, 0.1);
  color: var(--android-green);
}

.setting-item .value {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.setting-item .input,
.setting-item .select {
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-size: 14px;
  background: white;
  min-width: 100px;
}

.setting-item .checkbox {
  width: 20px;
  height: 20px;
  cursor: pointer;
}

.settings-actions {
  padding: 16px 20px;
  border-top: 1px solid var(--border-color);
}

.apply-btn {
  width: 100%;
  padding: 12px;
  background: var(--android-blue);
  color: white;
  border: none;
  border-radius: var(--radius-md);
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  min-height: 44px;
}

.apply-btn:hover {
  background: var(--android-blue-hover);
}
</style>
