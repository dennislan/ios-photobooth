<template>
  <div class="flex flex-col h-full gap-4 p-4">
    <!-- 预览区域 -->
    <div class="flex-1 flex flex-col bg-white rounded-lg border border-border-color overflow-hidden min-h-[360px]">
      <!-- 未连接：引导连接 -->
      <div
        v-if="!isCameraRunning"
        class="flex-1 flex flex-col items-center justify-center text-center p-8"
      >
        <CameraOutlined class="text-5xl text-text-tertiary mb-4" />
        <p class="text-lg font-medium text-text-primary mb-2">连接 iPhone</p>
        <p class="text-sm text-text-secondary mb-6">通过 USB 连接 iPhone 作为外接相机</p>
        <a-button
          v-if="!cameraStore.state.deviceId && !cameraStore.showFallbackOption"
          type="primary"
          size="large"
          :loading="cameraStore.connecting"
          @click="cameraStore.connect"
        >
          <template #icon><ScanOutlined /></template>
          连接手机
        </a-button>
        <div v-if="!cameraStore.state.deviceId && cameraStore.showFallbackOption" class="flex flex-col items-center gap-3">
          <p class="text-sm text-text-secondary mb-1">未检测到 iPhone 设备</p>
          <a-button
            type="primary"
            size="large"
            :loading="cameraStore.connecting"
            @click="cameraStore.startBuiltInCamera"
          >
            <template #icon><CameraOutlined /></template>
            使用 Mac 内置摄像头
          </a-button>
          <a-button
            v-if="cameraStore.hasLibMobileDevice"
            type="default"
            size="large"
            @click="retryConnection"
          >
            我有 iPhone，重新检测
          </a-button>
        </div>
        <div v-else-if="cameraStore.state.deviceId" class="flex flex-col items-center gap-3">
          <a-tag color="blue">设备: {{ cameraStore.state.deviceId.slice(0, 8) }}…</a-tag>
          <a-button type="primary" size="large" :loading="cameraStore.connecting" @click="cameraStore.connect">
            <template #icon><ScanOutlined /></template>
            重新连接
          </a-button>
        </div>
      </div>

      <!-- 已连接：MJPEG 预览 -->
      <div v-else class="relative flex-1 flex items-center justify-center bg-black">
        <img
          :src="mjpegUrl"
          class="w-full h-full object-contain"
          :style="{ filter: previewFilter }"
          alt="相机预览"
        />
        <a-badge
          :status="cameraStore.state.useBuiltInCamera ? 'warning' : 'processing'"
          :text="cameraStore.state.useBuiltInCamera ? '内置摄像头' : '预览中'"
          class="absolute top-3 right-3"
        />
      </div>
    </div>

    <!-- 控制栏：模板 + 滤镜 -->
    <div class="flex flex-wrap items-center gap-4 px-2">
      <div class="flex items-center gap-2">
        <span class="text-sm font-medium text-text-secondary">模板</span>
        <a-radio-group
          :value="captureStore.layoutId"
          button-style="solid"
          size="small"
          @change="(e: any) => captureStore.setLayout(e.target.value)"
        >
          <a-radio-button v-for="l in layouts" :key="l.id" :value="l.id">
            {{ l.name }}
          </a-radio-button>
        </a-radio-group>
      </div>

      <div class="flex items-center gap-2">
        <span class="text-sm font-medium text-text-secondary">滤镜</span>
        <a-radio-group
          :value="captureStore.filter"
          button-style="solid"
          size="small"
          @change="(e: any) => captureStore.setFilter(e.target.value)"
        >
          <a-radio-button v-for="f in filterKeys" :key="f" :value="f">
            {{ filterLabels[f] }}
          </a-radio-button>
        </a-radio-group>
      </div>
    </div>

    <!-- 拍照按钮 -->
    <div class="text-center">
      <a-button
        type="primary"
        size="large"
        shape="round"
        :disabled="!isCameraRunning || captureStore.photos.length >= captureStore.maxPhotos"
        :loading="capturing"
        class="!bg-[#34C759] !border-[#34C759] hover:!bg-[#2DB84E] hover:!border-[#2DB84E] !px-10 !h-12"
        @click="takePhoto"
      >
        <template #icon><CameraOutlined /></template>
        {{ capturing ? '拍摄中...' : '拍照' }}
        <span class="ml-2 opacity-70">({{ captureStore.photos.length }}/{{ captureStore.maxPhotos }})</span>
      </a-button>
    </div>

    <!-- 已拍照片缩略图条 -->
    <div
      v-if="captureStore.photos.length > 0"
      class="flex items-center gap-2 overflow-x-auto pb-1"
    >
      <div
        v-for="(photo, idx) in captureStore.photos"
        :key="idx"
        class="relative w-20 h-20 rounded-lg overflow-hidden flex-shrink-0 border border-border-color group"
      >
        <img :src="photo.dataUrl" class="w-full h-full object-cover" alt="已拍照片" />
        <button
          class="absolute top-1 right-1 w-5 h-5 bg-black/60 text-white rounded-full flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity hover:bg-danger"
          @click="removePhoto(idx)"
          aria-label="删除照片"
        >
          <CloseOutlined style="font-size: 11px;" />
        </button>
        <span class="absolute bottom-1 left-1 px-1.5 py-0.5 bg-black/60 text-white text-[10px] rounded">
          {{ idx + 1 }}
        </span>
      </div>

      <!-- 下一步按钮 -->
      <a-button
        type="primary"
        size="large"
        class="flex-shrink-0 ml-auto"
        @click="$emit('complete')"
      >
        下一步
        <template #icon><ArrowRightOutlined /></template>
      </a-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  CameraOutlined,
  ScanOutlined,
  CloseOutlined,
  ArrowRightOutlined,
} from '@ant-design/icons-vue';
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { message } from 'ant-design-vue';
import {
  useCaptureStore,
  FILTER_CSS,
  FILTER_LABELS,
  LAYOUTS,
  type FilterType,
} from '../stores/capture';
import { useCameraStore } from '../stores/camera';

defineEmits<{
  complete: [];
}>();

const captureStore = useCaptureStore();
const cameraStore = useCameraStore();

const capturing = ref(false);
const mjpegUrl = 'http://127.0.0.1:27183';

const layouts = LAYOUTS;
const filterKeys = Object.keys(FILTER_LABELS) as FilterType[];
const filterLabels = FILTER_LABELS;

const isCameraRunning = computed(() => cameraStore.isConnected);
const previewFilter = computed(() => FILTER_CSS[captureStore.filter]);

// 切换到拍照 tab 后自动连接 iPhone（如果尚未连接）
onMounted(() => {
  if (!cameraStore.isConnected) {
    cameraStore.connect();
  }
});

/** 用户点击"我有 iPhone，重新检测" */
function retryConnection() {
  cameraStore.showFallbackOption = false;
  cameraStore.connect();
}

/** 拍照 */
async function takePhoto() {
  capturing.value = true;
  try {
    const devId = cameraStore.state.deviceId;
    // 内置摄像头模式下 deviceId 为空，但仍可拍照
    if (!devId && !cameraStore.state.useBuiltInCamera) {
      message.warning('请先连接设备');
      return;
    }
    const photoPath = (await invoke('capture_photo', { deviceId: devId })) as string;
    const base64 = (await invoke('read_photo', { filename: photoPath })) as string;
    captureStore.addPhoto({
      dataUrl: `data:image/jpeg;base64,${base64}`,
      filePath: photoPath,
    });
    message.success('拍照成功');
  } catch (e) {
    message.error(e instanceof Error ? e.message : String(e));
  } finally {
    capturing.value = false;
  }
}

function removePhoto(index: number) {
  captureStore.removePhoto(index);
}
</script>
