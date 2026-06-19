<template>
  <a-modal :open="visible" title="设置" :footer="null" width="500px" class="settings-modal" @cancel="$emit('close')">
    <a-form layout="horizontal" :label-col="{ span: 8 }" :wrapper-col="{ span: 16 }">
      <a-divider orientation="left" orientation-size="small">设备状态</a-divider>
      <a-row :gutter="16" class="mb-3">
        <a-col :span="10"><span class="text-text-secondary text-sm">连接状态</span></a-col>
        <a-col :span="14">
          <a-badge :status="cameraStore.state.connected ? 'success' : 'default'" :text="cameraStore.state.connected ? '已连接' : '未连接'" />
        </a-col>
      </a-row>
      <a-row :gutter="16" class="mb-3">
        <a-col :span="10"><span class="text-text-secondary text-sm">设备 ID</span></a-col>
        <a-col :span="14"><span class="text-sm">{{ cameraStore.state.deviceId || '无' }}</span></a-col>
      </a-row>

      <a-divider orientation="left" orientation-size="small">打印设置</a-divider>
      <a-form-item label="纸张大小">
        <a-select v-model:value="localPaperSize">
          <a-select-option v-for="p in paperSizes" :key="p.value" :value="p.value">{{ p.label }}</a-select-option>
        </a-select>
      </a-form-item>
      <a-form-item label="色彩模式">
        <a-select v-model:value="localColorMode">
          <a-select-option v-for="c in colorModes" :key="c.value" :value="c.value">{{ c.label }}</a-select-option>
        </a-select>
      </a-form-item>

      <a-divider orientation="left" orientation-size="small">关于</a-divider>
      <a-row :gutter="16" class="mb-3">
        <a-col :span="10"><span class="text-text-secondary text-sm">版本</span></a-col>
        <a-col :span="14"><span class="text-sm">v{{ version }}</span></a-col>
      </a-row>
    </a-form>
    <template #footer>
      <a-button type="primary" @click="applySettings">应用设置</a-button>
    </template>
  </a-modal>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useCameraStore } from '../stores/camera';
import { useCaptureStore, PAPER_SIZES, COLOR_MODES } from '../stores/capture';

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: [] }>();

const cameraStore = useCameraStore();
const captureStore = useCaptureStore();
const version = '1.0.0';

const paperSizes = PAPER_SIZES;
const colorModes = COLOR_MODES;

const localPaperSize = ref('4x6');
const localColorMode = ref('color');

watch(() => props.visible, (val) => {
  if (val) {
    localPaperSize.value = captureStore.paperSize;
    localColorMode.value = captureStore.colorMode;
  }
});

function applySettings() {
  captureStore.paperSize = localPaperSize.value;
  captureStore.colorMode = localColorMode.value;
  localStorage.setItem('photobooth-settings', JSON.stringify({
    paperSize: localPaperSize.value,
    colorMode: localColorMode.value,
  }));
  emit('close');
}
</script>

<style scoped>
.settings-modal :deep(.ant-modal-body) { padding: 20px; }
.settings-modal :deep(.ant-modal-header) { padding: 16px 20px; }
.settings-modal :deep(.ant-divider-inner-text) { font-size: 16px; font-weight: 600; color: var(--color-primary, #007AFF); }
.settings-modal :deep(.ant-form-item-label) { padding-bottom: 4px; }
</style>
