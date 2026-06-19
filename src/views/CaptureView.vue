<template>
  <div class="flex flex-col h-full gap-4 p-4">
    <!-- 预览区域 -->
    <a-card :bordered="false" class="flex-1 !mb-0">
      <template #default>
        <div v-if="!isCameraRunning" class="flex flex-col items-center justify-center h-full min-h-[400px] text-center">
          <a-empty description="连接 iPhone" class="!justify-start">
            <template #description>
              <div>
                <p class="text-lg font-medium text-text-primary mb-2">连接 iPhone</p>
                <div v-if="deviceId" class="inline-flex items-center gap-2 px-3 py-2 rounded-full bg-[rgba(0,122,255,0.1)] text-sm font-medium mb-4">
                  <CheckCircleOutlined />
                  设备: {{ deviceId }}
                </div>
                <br>
                <a-button type="primary" size="large" @click="startCamera">
                  <template #icon>
                    <ScanOutlined />
                  </template>
                  连接手机
                </a-button>
              </div>
            </template>
          </a-empty>
        </div>
        <img
          v-else
          :src="mjpegUrl"
          class="w-full h-full object-contain"
          :style="{ filter: previewFilter }"
          alt="iPhone 相机预览"
        />
      </template>
      <template #extra v-if="isCameraRunning">
        <a-badge status="processing" text="预览中" />
      </template>
    </a-card>

    <!-- 模板选择 -->
    <div class="flex items-center gap-3">
      <span class="text-sm font-medium text-text-secondary flex-shrink-0">模板</span>
      <a-radio-group :value="captureStore.templateId" button-style="solid" @change="(e: any) => captureStore.setTemplate(e.target.value)">
        <a-radio-button v-for="tpl in templates" :key="tpl.id" :value="tpl.id">{{ tpl.name }}</a-radio-button>
      </a-radio-group>
    </div>

    <!-- 滤镜选择 -->
    <div class="flex items-center gap-3">
      <span class="text-sm font-medium text-text-secondary flex-shrink-0">滤镜</span>
      <a-radio-group :value="captureStore.filter" button-style="solid" size="small" @change="(e: any) => captureStore.setFilter(e.target.value)">
        <a-radio-button v-for="f in filterOptions" :key="f" :value="f">{{ filterLabels[f] }}</a-radio-button>
      </a-radio-group>
    </div>

    <!-- 拍照按钮 -->
    <div class="text-center">
      <a-button
        type="primary"
        size="large"
        :disabled="!isCameraRunning || captureStore.photos.length >= captureStore.maxPhotos"
        :loading="capturing"
        @click="takePhoto"
        class="!bg-[#34C759] !border-[#34C759] hover:!bg-[#2DB84E] hover:!border-[#2DB84E]"
      >
        <template #icon>
          <CameraOutlined />
        </template>
        {{ capturing ? '拍摄中...' : '拍照' }} ({{ captureStore.photos.length }}/{{ captureStore.maxPhotos }})
      </a-button>
    </div>

    <!-- 照片预览 -->
    <div
      v-if="captureStore.photos.length > 0"
      class="flex gap-2 overflow-x-auto pb-2"
      role="list"
      aria-label="已拍摄照片"
    >
      <div
        v-for="(photo, idx) in captureStore.photos"
        :key="idx"
        class="relative w-20 h-20 rounded-md overflow-hidden cursor-pointer flex-shrink-0 border-2 transition-all"
        :class="captureStore.selectedIndex === idx ? 'border-primary ring-2 ring-[rgba(0,122,255,0.2)]' : 'border-transparent'"
        @click="captureStore.selectPhoto(idx)"
        role="listitem"
        tabindex="0"
        :aria-label="`照片 ${idx + 1}${photo.isLive ? ', Live Photo' : ''}`"
      >
        <a-image
          :src="photo.dataUrl"
          :preview="false"
          class="!w-full !h-full"
          style="object-fit: cover;"
        />
        <button
          class="absolute top-1 right-1 w-6 h-6 bg-[rgba(0,0,0,0.6)] text-white rounded-full flex items-center justify-center hover:bg-[rgba(255,59,48,0.8)] transition-colors"
          @click.stop="removePhoto(idx)"
          aria-label="删除照片"
        >
          <CloseOutlined style="font-size: 12px;" />
        </button>
        <span
          v-if="photo.isLive"
          class="absolute bottom-1 left-1 px-1.5 py-0.5 bg-[rgba(0,0,0,0.7)] text-white text-[10px] font-bold tracking-wider rounded"
        >
          LIVE
        </span>
      </div>
    </div>

    <!-- 完成按钮 -->
    <a-button
      v-if="captureStore.photos.length > 0"
      type="primary"
      size="large"
      block
      @click="$emit('complete')"
    >
      <template #icon>
        <ArrowRightOutlined />
      </template>
      下一步
    </a-button>
  </div>
</template>

<script setup lang="ts">
import {
  CameraOutlined,
  ScanOutlined,
  CloseOutlined,
  CheckCircleOutlined,
  ArrowRightOutlined,
} from "@ant-design/icons-vue"
import { ref, computed, onUnmounted } from "vue"
import { useCaptureStore, FILTER_CSS, FILTER_LABELS, TEMPLATES, type FilterType } from "../stores/capture"
import { useCameraStore } from "../stores/camera"

defineEmits<{
  complete: []
}>()

const captureStore = useCaptureStore()
const cameraStore = useCameraStore()
const deviceId = ref("")
const capturing = ref(false)
const mjpegUrl = "http://127.0.0.1:27183"

const templates = TEMPLATES
const filterOptions = Object.keys(FILTER_LABELS) as FilterType[]
const filterLabels = FILTER_LABELS

const isCameraRunning = computed(() => cameraStore.isConnected)
const previewFilter = computed(() => FILTER_CSS[captureStore.filter])

async function startCamera() {
  try {
    const { invoke } = await import("@tauri-apps/api/core")
    const devices: string[] = await invoke("get_device_list")
    if (devices.length === 0) {
      alert("未检测到设备，请检查 USB 连接")
      return
    }
    deviceId.value = devices[0]
    await invoke("start_camera_stream", { deviceId: devices[0] })
    cameraStore.setState({
      connected: true,
      running: true,
      deviceId: devices[0],
    })
  } catch (e) {
    alert(e instanceof Error ? e.message : String(e))
  }
}

async function takePhoto() {
  capturing.value = true
  try {
    const { invoke } = await import("@tauri-apps/api/core")
    const devId = cameraStore.state.deviceId
    if (!devId) {
      alert("请先连接设备")
      return
    }
    const photoPath: string = await invoke("take_photo", { deviceId: devId })
    const thumbnail: string = await invoke("get_photo_thumbnail", {
      filename: photoPath,
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
  } finally {
    capturing.value = false
  }
}

function removePhoto(index: number) {
  captureStore.removePhoto(index)
}

onUnmounted(async () => {
  try {
    const { invoke } = await import("@tauri-apps/api/core")
    await invoke("stop_camera_stream")
    cameraStore.reset()
  } catch {
    // ignore errors on cleanup
  }
})
</script>
