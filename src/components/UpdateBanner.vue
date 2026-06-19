<template>
  <div v-if="isVisible" class="update-banner" :class="{ 'banner-error': store.status === 'error' }">
    <div class="banner-content">
      <!-- Checking -->
      <div v-if="store.status === 'checking'" class="banner-row">
        <span class="banner-icon">
          <LoadingOutlined />
        </span>
        <span class="banner-text">正在检查更新...</span>
      </div>

      <!-- Up to date -->
      <div v-if="store.status === 'up-to-date'" class="banner-row">
        <span class="banner-icon">
          <CheckCircleOutlined />
        </span>
        <span class="banner-text">当前已是最新版本 v{{ store.currentVersion }}</span>
      </div>

      <!-- Update available -->
      <div v-if="store.status === 'available'" class="banner-row">
        <span class="banner-icon">
          <ArrowDownOutlined />
        </span>
        <span class="banner-text">
          发现新版本 v{{ store.latestVersion }}
          <template v-if="!store.mandatory">
            · 点击立即更新
          </template>
        </span>
        <a-button
          v-if="!store.mandatory"
          type="primary"
          size="small"
          @click="store.applyUpdate()"
          class="update-btn"
        >
          立即更新
        </a-button>
        <a-button
          v-if="store.mandatory"
          type="primary"
          size="small"
          danger
          @click="store.applyUpdate()"
          class="update-btn"
        >
          必须更新
        </a-button>
        <a-button
          v-if="!store.mandatory"
          size="small"
          @click="dismissBanner"
          class="dismiss-btn"
        >
          稍后
        </a-button>
      </div>

      <!-- Downloading -->
      <div v-if="store.status === 'downloading'" class="banner-row">
        <span class="banner-icon">
          <LoadingOutlined />
        </span>
        <div class="progress-wrap">
          <a-progress
            :percent="store.progress"
            :show-info="true"
            size="small"
            status="active"
          />
        </div>
        <a-button size="small" @click="dismissBanner" class="dismiss-btn">
          后台下载
        </a-button>
      </div>

      <!-- Error -->
      <div v-if="store.status === 'error'" class="banner-row">
        <span class="banner-icon">
          <ExclamationCircleOutlined />
        </span>
        <span class="banner-text">{{ store.error || "更新失败" }}</span>
        <a-button size="small" @click="store.checkForUpdates()" class="retry-btn">
          重试
        </a-button>
        <a-button size="small" @click="dismissBanner" class="dismiss-btn">
          关闭
        </a-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue"
import { useUpdateStore } from "../stores/update"
import { LoadingOutlined, CheckCircleOutlined, ArrowDownOutlined, ExclamationCircleOutlined } from "@ant-design/icons-vue"

const store = useUpdateStore()
const isVisible = ref(true)

function dismissBanner() {
  if (store.status === "available" && !store.mandatory) {
    // Keep the banner hidden for this session
    isVisible.value = false
  }
}

// Reset visibility when a new update becomes available
watch(
  () => store.status,
  (newStatus) => {
    if (newStatus === "available") {
      isVisible.value = true
    }
  },
)
</script>

<style scoped>
.update-banner {
  background: var(--color-primary, #007aff);
  color: #fff;
  padding: 8px 16px;
  font-size: 13px;
  z-index: 1000;
}

.update-banner.banner-error {
  background: #ff4d4f;
}

.banner-content {
  max-width: 1200px;
  margin: 0 auto;
}

.banner-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.banner-icon {
  font-size: 14px;
  flex-shrink: 0;
}

.banner-text {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.update-btn {
  background: rgba(255, 255, 255, 0.2);
  border-color: rgba(255, 255, 255, 0.4);
  color: #fff;
  flex-shrink: 0;
}

.update-btn:hover {
  background: rgba(255, 255, 255, 0.3);
  border-color: #fff;
}

.dismiss-btn,
.retry-btn {
  flex-shrink: 0;
  color: rgba(255, 255, 255, 0.8);
  border-color: rgba(255, 255, 255, 0.3);
}

.dismiss-btn:hover,
.retry-btn:hover {
  color: #fff;
  border-color: #fff;
}

.progress-wrap {
  flex: 1;
  min-width: 120px;
  max-width: 300px;
}
</style>
