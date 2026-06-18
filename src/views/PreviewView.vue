<template>
  <div class="preview-view">
    <div class="preview-header">
      <h2>预览与打印</h2>
      <button class="btn btn-secondary back-btn" @click="$emit('back')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="btn-icon-svg">
          <polyline points="15 18 9 12 15 6"/>
        </svg>
        返回
      </button>
    </div>

    <div class="preview-content">
      <!-- 预览区域 -->
      <div class="preview-area">
        <div v-if="selectedPhotos.length === 0" class="empty-state">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="empty-icon">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
            <circle cx="8.5" cy="8.5" r="1.5"/>
            <polyline points="21 15 16 10 5 21"/>
          </svg>
          <p>请先选择照片</p>
        </div>
        
        <div v-else class="preview-canvas-wrapper">
          <canvas ref="canvasRef" class="preview-canvas"></canvas>
          <div class="preview-actions">
            <button class="btn btn-secondary" @click="resetZoom">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="btn-icon-svg">
                <circle cx="11" cy="11" r="8"/>
                <line x1="21" y1="21" x2="16.65" y2="16.65"/>
                <line x1="11" y1="8" x2="11" y2="14"/>
                <line x1="8" y1="11" x2="14" y2="11"/>
              </svg>
              重置缩放
            </button>
          </div>
        </div>
      </div>

      <!-- 打印选项 -->
      <div class="print-options">
        <h3 class="section-title">打印设置</h3>
        <div class="options-grid">
          <div class="option-item">
            <label>纸张大小</label>
            <select v-model="paperSize" class="select">
              <option value="4x6">4x6 英寸</option>
              <option value="5x7">5x7 英寸</option>
              <option value="6x8">6x8 英寸</option>
            </select>
          </div>
          <div class="option-item">
            <label>打印模式</label>
            <select v-model="printMode" class="select">
              <option value="color">彩色</option>
              <option value="bw">黑白</option>
            </select>
          </div>
          <div class="option-item">
            <label>份数</label>
            <input type="number" v-model.number="copies" min="1" max="10" class="input" />
          </div>
        </div>
      </div>
    </div>

    <div class="preview-footer">
      <button class="btn btn-success print-btn" @click="printImage" :disabled="printing">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="btn-icon-svg">
          <polyline points="6 9 6 2 18 2 18 9"/>
          <path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"/>
          <rect x="6" y="14" width="12" height="8"/>
        </svg>
        {{ printing ? '打印中...' : '打印照片' }}
      </button>
      <button class="btn btn-secondary download-btn" @click="downloadImage">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="btn-icon-svg">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="7 10 12 15 17 10"/>
          <line x1="12" y1="15" x2="12" y2="3"/>
        </svg>
        下载
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useCaptureStore } from '../stores/capture';

defineEmits<{
  back: [];
}>();

const captureStore = useCaptureStore();
const canvasRef = ref<HTMLCanvasElement | null>(null);
const paperSize = ref('4x6');
const printMode = ref('color');
const copies = ref(1);
const printing = ref(false);

const selectedPhotos = computed(() => captureStore.photos);

onMounted(() => {
  renderPreview();
});

watch([selectedPhotos], () => {
  renderPreview();
});

function renderPreview() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  
  const ctx = canvas.getContext('2d');
  if (!ctx) return;
  
  canvas.width = 800;
  canvas.height = 600;
  
  // Background
  ctx.fillStyle = '#FFFFFF';
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  
  // Render photos
  selectedPhotos.value.forEach((photo, idx) => {
    const img = new Image();
    img.onload = () => {
      // Simple grid layout
      const cols = 2;
      const rows = Math.ceil(selectedPhotos.value.length / cols);
      const cellWidth = canvas.width / cols;
      const cellHeight = canvas.height / rows;
      const x = (idx % cols) * cellWidth;
      const y = Math.floor(idx / cols) * cellHeight;
      
      // Cover fit
      const scale = Math.max(cellWidth / img.width, cellHeight / img.height);
      const dw = img.width * scale;
      const dh = img.height * scale;
      const dx = x + (cellWidth - dw) / 2;
      const dy = y + (cellHeight - dh) / 2;
      
      ctx.drawImage(img, dx, dy, dw, dh);
    };
    img.src = photo.dataUrl;
  });
}

async function printImage() {
  printing.value = true;
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('print_image', {
      filePath: '',
      copies: copies.value,
      colorMode: printMode.value,
    });
    alert('打印成功！');
  } catch (e) {
    alert(e instanceof Error ? e.message : String(e));
  } finally {
    printing.value = false;
  }
}

async function downloadImage() {
  try {
    const { save } = await import('@tauri-apps/plugin-dialog');
    
    const selectedPath = await save({
      filters: [{ name: 'PNG', extensions: ['png'] }],
    });
    
    if (selectedPath) {
      const canvas = canvasRef.value;
      if (canvas) {
        const blob = await new Promise<Blob | null>((resolve) => {
          canvas.toBlob(resolve, 'image/png');
        });
        
        if (blob) {
          alert(`图片已准备下载: ${selectedPath}`);
        }
      }
    }
  } catch (e) {
    alert(e instanceof Error ? e.message : String(e));
  }
}

function resetZoom() {
  renderPreview();
}
</script>

<style scoped>
.preview-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  background: white;
  border-bottom: 1px solid var(--border-color);
}

.preview-header h2 {
  font-size: 20px;
  font-weight: 600;
  margin: 0;
}

.back-btn {
  min-height: 40px;
}

.btn-icon-svg {
  width: 16px;
  height: 16px;
}

.preview-content {
  flex: 1;
  display: flex;
  gap: 24px;
  padding: 24px;
  overflow: hidden;
}

.preview-area {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
}

.empty-icon {
  width: 64px;
  height: 64px;
  margin-bottom: 16px;
}

.preview-canvas-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: white;
  border-radius: var(--radius-lg);
  border: 1px solid var(--border-color);
  overflow: hidden;
}

.preview-canvas {
  flex: 1;
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.preview-actions {
  padding: 12px;
  border-top: 1px solid var(--border-color);
}

.print-options {
  width: 300px;
  background: white;
  border-radius: var(--radius-lg);
  border: 1px solid var(--border-color);
  padding: 20px;
  overflow-y: auto;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 16px;
}

.options-grid {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.option-item label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary);
  margin-bottom: 6px;
}

.option-item .input,
.option-item .select {
  width: 100%;
  padding: 10px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-size: 14px;
  background: white;
}

.preview-footer {
  display: flex;
  gap: 12px;
  padding: 16px 24px;
  background: white;
  border-top: 1px solid var(--border-color);
}

.print-btn,
.download-btn {
  flex: 1;
  padding: 14px;
  font-size: 16px;
  min-height: 48px;
}

@media (max-width: 768px) {
  .preview-content {
    flex-direction: column;
  }
  
  .print-options {
    width: 100%;
  }
}
</style>
