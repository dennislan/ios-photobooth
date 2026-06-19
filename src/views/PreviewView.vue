<template>
  <div class="flex flex-col h-full">
    <!-- Header -->
    <div class="flex items-center justify-between px-6 py-3 bg-white border-b border-border-color">
      <h2 class="text-lg font-semibold m-0">预览与打印</h2>
      <a-button @click="$emit('back')">
        <template #icon><ArrowLeftOutlined /></template>
        返回
      </a-button>
    </div>

    <!-- Content -->
    <div class="flex-1 flex gap-6 p-6 overflow-hidden">
      <!-- 预览区域 -->
      <div class="flex-1 flex flex-col">
        <a-empty
          v-if="layoutPhotos.length === 0"
          description="请先选择照片"
          class="!flex !items-center !justify-center !h-full"
        />
        <div v-else class="flex-1 flex flex-col bg-white rounded-lg border border-border-color overflow-hidden">
          <div class="flex-1 flex items-center justify-center p-4 overflow-hidden">
            <canvas ref="canvasRef" class="max-w-full max-h-full object-contain shadow-lg rounded"></canvas>
          </div>
          <div class="p-3 border-t border-border-color flex items-center gap-3">
            <span class="text-sm text-text-secondary">模板: {{ currentTemplateName }}</span>
            <span class="text-sm text-text-secondary">|</span>
            <span class="text-sm text-text-secondary">滤镜: {{ filterLabel }}</span>
            <span class="text-sm text-text-secondary">|</span>
            <span class="text-sm text-text-secondary">{{ layoutPhotos.length }} 张照片</span>
          </div>
        </div>
      </div>

      <!-- 打印选项 -->
      <a-card title="打印设置" :bordered="false" class="w-[300px] flex-shrink-0">
        <a-form layout="vertical">
          <a-form-item label="纸张大小">
            <a-select v-model:value="captureStore.paperSize">
              <a-select-option value="4x6">4x6 英寸</a-select-option>
              <a-select-option value="5x7">5x7 英寸</a-select-option>
              <a-select-option value="6x8">6x8 英寸</a-select-option>
              <a-select-option value="A4">A4</a-select-option>
            </a-select>
          </a-form-item>
          <a-form-item label="打印模式">
            <a-select v-model:value="captureStore.printMode">
              <a-select-option value="color">彩色</a-select-option>
              <a-select-option value="bw">黑白</a-select-option>
            </a-select>
          </a-form-item>
          <a-form-item label="份数">
            <a-input-number v-model:value="captureStore.copies" :min="1" :max="20" style="width: 100%;" />
          </a-form-item>
          <a-divider style="margin: 8px 0;" />
          <a-form-item label="滤镜调整">
            <a-select :value="captureStore.filter" @change="(v: any) => captureStore.setFilter(v)">
              <a-select-option v-for="f in filterOptions" :key="f" :value="f">{{ FILTER_LABELS[f] }}</a-select-option>
            </a-select>
          </a-form-item>
        </a-form>
      </a-card>
    </div>

    <!-- Footer -->
    <div class="px-6 py-4 bg-white border-t border-border-color flex gap-3">
      <a-button
        type="primary"
        size="large"
        :loading="printing"
        class="flex-1 !py-3.5 !text-base"
        style="background: #34C759; border-color: #34C759;"
        @click="printImage"
      >
        <template #icon><PrinterOutlined /></template>
        {{ printing ? '打印中...' : '打印照片' }}
      </a-button>
      <a-button
        size="large"
        class="flex-1 !py-3.5 !text-base"
        :loading="downloading"
        @click="downloadImage"
      >
        <template #icon><DownloadOutlined /></template>
        下载
      </a-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  ArrowLeftOutlined,
  PrinterOutlined,
  DownloadOutlined,
} from "@ant-design/icons-vue"
import { ref, computed, onMounted, watch, nextTick } from 'vue'
import {
  useCaptureStore,
  FILTER_CSS,
  FILTER_LABELS,
  TEMPLATES,
  type FilterType,
} from '../stores/capture'

defineEmits<{
  back: []
}>()

const captureStore = useCaptureStore()
const canvasRef = ref<HTMLCanvasElement | null>(null)
const printing = ref(false)
const downloading = ref(false)

const filterOptions = Object.keys(FILTER_LABELS) as FilterType[]

const layoutPhotos = computed(() => captureStore.photosForLayout)
const filterLabel = computed(() => FILTER_LABELS[captureStore.filter])

const currentTemplateName = computed(() => {
  const tpl = TEMPLATES.find((t) => t.id === captureStore.templateId)
  return tpl ? tpl.name : '默认'
})

// 画布尺寸（高分辨率，用于打印）
function canvasDimensions(): { w: number; h: number } {
  switch (captureStore.templateId) {
    case 'single':
      return { w: 1200, h: 1800 }
    case 'strip':
      return { w: 900, h: 1800 }
    case 'grid4':
    default:
      return { w: 1200, h: 1800 }
  }
}

onMounted(() => {
  nextTick(() => renderPreview())
})

watch(
  [layoutPhotos, () => captureStore.filter, () => captureStore.templateId],
  () => {
    nextTick(() => renderPreview())
  }
)

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.crossOrigin = 'anonymous'
    img.onload = () => resolve(img)
    img.onerror = reject
    img.src = src
  })
}

async function renderPreview() {
  const canvas = canvasRef.value
  if (!canvas || layoutPhotos.value.length === 0) return

  const { w, h } = canvasDimensions()
  canvas.width = w
  canvas.height = h

  const ctx = canvas.getContext('2d')
  if (!ctx) return

  // 背景
  ctx.fillStyle = '#FFFFFF'
  ctx.fillRect(0, 0, w, h)

  // 应用滤镜
  const filterCss = FILTER_CSS[captureStore.filter]
  ctx.filter = filterCss === 'none' ? 'none' : filterCss

  const template = TEMPLATES.find((t) => t.id === captureStore.templateId)
  const cols = template?.cols ?? 2
  const rows = template?.rows ?? 2
  const padding = 40
  const gap = 20

  const cellW = (w - padding * 2 - gap * (cols - 1)) / cols
  const cellH = (h - padding * 2 - gap * (rows - 1)) / rows

  // 加载所有照片
  const images: HTMLImageElement[] = []
  for (const photo of layoutPhotos.value) {
    try {
      const img = await loadImage(photo.dataUrl)
      images.push(img)
    } catch {
      // 跳过加载失败的
    }
  }

  // 绘制每张照片 (object-fit: cover)
  images.forEach((img, idx) => {
    if (idx >= cols * rows) return
    const col = idx % cols
    const row = Math.floor(idx / cols)
    const x = padding + col * (cellW + gap)
    const y = padding + row * (cellH + gap)

    // cover 算法
    const imgRatio = img.width / img.height
    const cellRatio = cellW / cellH
    let dw: number, dh: number
    if (imgRatio > cellRatio) {
      dh = cellH
      dw = cellH * imgRatio
    } else {
      dw = cellW
      dh = cellW / imgRatio
    }
    const dx = x + (cellW - dw) / 2
    const dy = y + (cellH - dh) / 2

    ctx.save()
    // 裁剪到单元格
    ctx.beginPath()
    ctx.rect(x, y, cellW, cellH)
    ctx.clip()
    ctx.drawImage(img, dx, dy, dw, dh)
    ctx.restore()
  })

  // 重置滤镜
  ctx.filter = 'none'
}

async function canvasToBlob(quality = 0.92): Promise<Blob | null> {
  const canvas = canvasRef.value
  if (!canvas) return null
  return new Promise((resolve) => {
    canvas.toBlob(resolve, 'image/jpeg', quality)
  })
}

function blobToBase64(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(reader.result as string)
    reader.onerror = reject
    reader.readAsDataURL(blob)
  })
}

async function printImage() {
  if (layoutPhotos.value.length === 0) {
    alert('请先选择照片')
    return
  }
  printing.value = true
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const blob = await canvasToBlob(0.92)
    if (!blob) {
      alert('图片生成失败')
      return
    }
    const base64 = await blobToBase64(blob)
    const imagePath: string = await invoke('save_temp_image', { data: base64, ext: 'jpg' })
    await invoke('print_image', {
      imagePath,
      paperSize: captureStore.paperSize,
      colorMode: captureStore.printMode,
      copies: captureStore.copies,
    })
    alert(`打印成功！已发送 ${captureStore.copies} 份到打印机`)
  } catch (e) {
    alert(e instanceof Error ? e.message : String(e))
  } finally {
    printing.value = false
  }
}

async function downloadImage() {
  if (layoutPhotos.value.length === 0) {
    alert('请先选择照片')
    return
  }
  downloading.value = true
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const { save } = await import('@tauri-apps/plugin-dialog')

    const selectedPath = await save({
      defaultPath: `photobooth_${Date.now()}.jpg`,
      filters: [{ name: 'JPEG', extensions: ['jpg'] }],
    })

    if (!selectedPath) return

    const blob = await canvasToBlob(0.95)
    if (!blob) {
      alert('图片生成失败')
      return
    }
    const base64 = await blobToBase64(blob)
    await invoke('write_image_file', { path: selectedPath, data: base64 })
    alert(`已保存到: ${selectedPath}`)
  } catch (e) {
    alert(e instanceof Error ? e.message : String(e))
  } finally {
    downloading.value = false
  }
}
</script>
