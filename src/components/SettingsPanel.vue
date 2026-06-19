<template>
  <a-modal
    :open="visible"
    title="设置"
    :footer="null"
    :closable="{ onClose: $emit('close') }"
    width="500px"
    class="settings-modal"
    @cancel="$emit('close')"
  >
    <a-form layout="horizontal" :label-col="{ span: 8 }" :wrapper-col="{ span: 16 }">
      <!-- 设备设置 -->
      <a-divider orientation="left" orientation-size="small">设备设置</a-divider>
      <a-row :gutter="16" style="margin-bottom: 12px;">
        <a-col :span="10">
          <span style="color: var(--color-text-secondary, #636366); font-size: 14px;">设备连接</span>
        </a-col>
        <a-col :span="14">
          <a-badge
            :status="cameraStore.state.connected ? 'success' : 'default'"
            :text="cameraStore.state.connected ? '已连接' : '未连接'"
          />
        </a-col>
      </a-row>
      <a-row :gutter="16" style="margin-bottom: 12px;">
        <a-col :span="10">
          <span style="color: var(--color-text-secondary, #636366); font-size: 14px;">设备 ID</span>
        </a-col>
        <a-col :span="14">
          <span>{{ cameraStore.state.deviceId || '无' }}</span>
        </a-col>
      </a-row>

      <!-- 拍摄设置 -->
      <a-divider orientation="left" orientation-size="small">拍摄设置</a-divider>
      <a-form-item label="最大照片数">
        <a-input-number
          v-model:value="localMaxPhotos"
          :min="1"
          :max="10"
          style="width: 100%;"
        />
      </a-form-item>
      <a-form-item label="自动翻转">
        <a-switch v-model:checked="localAutoFlip" />
      </a-form-item>

      <!-- 打印设置 -->
      <a-divider orientation="left" orientation-size="small">打印设置</a-divider>
      <a-form-item label="纸张大小">
        <a-select v-model:value="localPaperSize">
          <a-select-option value="4x6">4x6 英寸</a-select-option>
          <a-select-option value="5x7">5x7 英寸</a-select-option>
          <a-select-option value="6x8">6x8 英寸</a-select-option>
        </a-select>
      </a-form-item>
      <a-form-item label="打印模式">
        <a-select v-model:value="localPrintMode">
          <a-select-option value="color">彩色</a-select-option>
          <a-select-option value="bw">黑白</a-select-option>
        </a-select>
      </a-form-item>

      <!-- 关于 -->
      <a-divider orientation="left" orientation-size="small">关于</a-divider>
      <a-row :gutter="16" style="margin-bottom: 12px;">
        <a-col :span="10">
          <span style="color: var(--color-text-secondary, #636366); font-size: 14px;">版本</span>
        </a-col>
        <a-col :span="14">
          <span>{{ updateStore.currentVersion || 'v1.0.0' }}</span>
        </a-col>
      </a-row>
      <a-row :gutter="16" style="margin-bottom: 12px;">
        <a-col :span="10">
          <span style="color: var(--color-text-secondary, #636366); font-size: 14px;">更新状态</span>
        </a-col>
        <a-col :span="14">
          <a-tag v-if="updateStore.status === 'up-to-date'" color="green">
            当前已是最新
          </a-tag>
          <a-tag v-else-if="updateStore.status === 'checking'" color="blue">
            正在检查...
          </a-tag>
          <a-tag v-else-if="updateStore.status === 'available'" color="orange">
            新版本 v{{ updateStore.latestVersion }} 可用
          </a-tag>
          <a-tag v-else-if="updateStore.status === 'downloading'" color="cyan">
            下载中 {{ updateStore.progress }}%
          </a-tag>
          <a-tag v-else-if="updateStore.status === 'error'" color="red">
            {{ updateStore.error || '检查失败' }}
          </a-tag>
          <a-tag v-else color="default">
            就绪
          </a-tag>
        </a-col>
      </a-row>
      <a-row :gutter="16">
        <a-col :span="24">
          <a-button
            type="primary"
            size="small"
            block
            @click="checkForUpdates"
            :loading="updateStore.status === 'checking'"
          >
            检查更新
          </a-button>
        </a-col>
      </a-row>
      <a-row v-if="updateStore.changelog" :gutter="16" style="margin-top: 12px;">
        <a-col :span="24">
          <div style="font-size: 12px; color: var(--color-text-secondary, #636366); white-space: pre-wrap; max-height: 120px; overflow-y: auto;">
            {{ updateStore.changelog }}
          </div>
        </a-col>
      </a-row>
    </a-form>

    <template #footer>
      <a-button type="primary" @click="applySettings">应用设置</a-button>
    </template>
  </a-modal>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from "vue"
import { useCameraStore } from "../stores/camera"
import { useUpdateStore } from "../stores/update"
import { useCaptureStore } from "../stores/capture"

const props = defineProps<{
  visible: boolean
}>()

defineEmits<{
  close: []
}>()

const cameraStore = useCameraStore()
const updateStore = useUpdateStore()
const captureStore = useCaptureStore()

// Local state synced with parent visibility
const localVisible = ref(false)

watch(
  () => props.visible,
  (val) => {
    localVisible.value = val
    if (val) {
      // 从 store 加载当前值
      localPaperSize.value = captureStore.paperSize
      localPrintMode.value = captureStore.printMode
      localMaxPhotos.value = captureStore.maxPhotos
    }
  },
)

const localMaxPhotos = ref(4)
const localAutoFlip = ref(false)
const localPaperSize = ref("4x6")
const localPrintMode = ref("color")

onMounted(() => {
  // Sync version from store
  updateStore.currentVersion = "1.0.0" // fallback, updated by checkForUpdates
})

async function checkForUpdates() {
  await updateStore.checkForUpdates()
}

function applySettings() {
  // 同步到 capture store
  captureStore.paperSize = localPaperSize.value
  captureStore.printMode = localPrintMode.value

  localStorage.setItem(
    "photobooth-settings",
    JSON.stringify({
      maxPhotos: localMaxPhotos.value,
      autoFlip: localAutoFlip.value,
      paperSize: localPaperSize.value,
      printMode: localPrintMode.value,
    }),
  )
  alert("设置已保存")
}
</script>

<style scoped>
.settings-modal :deep(.ant-modal-body) {
  padding: 20px;
}

.settings-modal :deep(.ant-modal-header) {
  padding: 16px 20px;
}

.settings-modal :deep(.ant-modal-footer) {
  padding: 16px 20px;
}

.settings-modal :deep(.ant-divider-inner-text) {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-primary, #007AFF);
}

.settings-modal :deep(.ant-form-item-label) {
  padding-bottom: 4px;
}
</style>
