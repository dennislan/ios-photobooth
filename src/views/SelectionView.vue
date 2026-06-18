<template>
  <div class="selection-view">
    <div class="selection-header">
      <h2>选择照片</h2>
      <button class="btn btn-secondary back-btn" @click="$emit('back')">
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          class="btn-icon-svg"
        >
          <polyline points="15 18 9 12 15 6" />
        </svg>
        返回
      </button>
    </div>

    <div class="selection-content">
      <!-- 模板选择 -->
      <div class="template-section">
        <h3 class="section-title">选择模板</h3>
        <div class="template-grid">
          <button
            v-for="tpl in templates"
            :key="tpl.id"
            class="template-card"
            :class="{ active: selectedTemplate === tpl.id }"
            @click="selectedTemplate = tpl.id"
            :aria-pressed="selectedTemplate === tpl.id"
          >
            <div
              class="template-preview"
              :style="{ backgroundColor: tpl.bgColor }"
            >
              <div class="preview-grid" :class="tpl.gridClass">
                <div
                  v-for="n in tpl.previewCount"
                  :key="n"
                  class="preview-slot"
                ></div>
              </div>
            </div>
            <span class="template-name">{{ tpl.name }}</span>
          </button>
        </div>
      </div>

      <!-- 照片网格 -->
      <div class="photo-section">
        <h3 class="section-title">
          已拍摄照片 ({{ photos.length }})
          <span class="photo-count" v-if="photos.length > 0">
            / {{ captureStore.maxPhotos }}
          </span>
        </h3>

        <div v-if="photos.length === 0" class="empty-state">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            class="empty-icon"
          >
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
            <circle cx="8.5" cy="8.5" r="1.5" />
            <polyline points="21 15 16 10 5 21" />
          </svg>
          <p>还没有拍摄照片哦</p>
        </div>

        <div class="photo-grid" v-else>
          <div
            v-for="(photo, idx) in photos"
            :key="idx"
            class="photo-card"
            :class="{ selected: selectedPhotos.includes(idx) }"
            @click="toggleSelect(idx)"
            role="checkbox"
            :aria-checked="selectedPhotos.includes(idx)"
            tabindex="0"
            @keydown.enter="toggleSelect(idx)"
            @keydown.space.prevent="toggleSelect(idx)"
          >
            <img :src="photo.dataUrl" alt="" loading="lazy" />
            <div class="photo-overlay">
              <span class="photo-number">{{ idx + 1 }}</span>
              <button
                class="delete-btn"
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
            </div>
            <div class="check-mark" v-if="selectedPhotos.includes(idx)">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="white"
                stroke-width="3"
              >
                <polyline points="20 6 9 17 4 12" />
              </svg>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="selection-footer">
      <button
        class="btn btn-primary confirm-btn"
        :disabled="selectedPhotos.length === 0"
        @click="$emit('confirm')"
      >
        确认选择 ({{ selectedPhotos.length }})
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue"
import { useCaptureStore } from "../stores/capture"

defineEmits<{
  confirm: []
  back: []
}>()

const captureStore = useCaptureStore()
const selectedTemplate = ref("default")
const selectedPhotos = ref<number[]>([])

const photos = computed(() => captureStore.photos)

const templates = [
  {
    id: "default",
    name: "默认",
    bgColor: "#F0F0F0",
    gridClass: "grid-2x2",
    previewCount: 4,
  },
  {
    id: "single",
    name: "单张",
    bgColor: "#E8F4FD",
    gridClass: "grid-1x1",
    previewCount: 1,
  },
  {
    id: "strip",
    name: "长条",
    bgColor: "#FFF4E6",
    gridClass: "grid-1x4",
    previewCount: 4,
  },
]

function toggleSelect(idx: number) {
  const pos = selectedPhotos.value.indexOf(idx)
  if (pos >= 0) {
    selectedPhotos.value.splice(pos, 1)
  } else {
    selectedPhotos.value.push(idx)
  }
}

function removePhoto(index: number) {
  captureStore.removePhoto(index)
  selectedPhotos.value = selectedPhotos.value.filter((i) => i !== index)
}
</script>

<style scoped>
.selection-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.selection-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  background: white;
  border-bottom: 1px solid var(--border-color);
}

.selection-header h2 {
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

.selection-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 16px;
}

.photo-count {
  font-weight: 400;
  color: var(--text-secondary);
}

.template-section,
.photo-section {
  margin-bottom: 32px;
}

.template-grid {
  display: flex;
  gap: 16px;
}

.template-card {
  flex: 1;
  border: 2px solid var(--border-color);
  border-radius: var(--radius-md);
  padding: 12px;
  cursor: pointer;
  transition: all var(--transition-base);
  background: transparent;
}

.template-card:hover {
  border-color: var(--android-blue);
}

.template-card.active {
  border-color: var(--android-blue);
  background: rgba(0, 122, 255, 0.05);
}

.template-preview {
  aspect-ratio: 1;
  border-radius: var(--radius-sm);
  display: flex;
  gap: 4px;
  padding: 4px;
  margin-bottom: 8px;
}

.preview-grid {
  display: grid;
  gap: 4px;
  width: 100%;
  height: 100%;
}

.grid-2x2 {
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr;
}

.grid-1x1 {
  grid-template-columns: 1fr;
  grid-template-rows: 1fr;
}

.grid-1x4 {
  grid-template-columns: 1fr;
  grid-template-rows: repeat(4, 1fr);
}

.preview-slot {
  background: rgba(0, 0, 0, 0.1);
  border-radius: 2px;
}

.template-name {
  display: block;
  text-align: center;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.empty-state {
  text-align: center;
  padding: 48px 24px;
  color: var(--text-tertiary);
}

.empty-icon {
  width: 48px;
  height: 48px;
  margin: 0 auto 16px;
  color: var(--text-tertiary);
}

.photo-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 16px;
}

.photo-card {
  position: relative;
  aspect-ratio: 1;
  border-radius: var(--radius-md);
  overflow: hidden;
  cursor: pointer;
  border: 2px solid var(--border-color);
  transition: all var(--transition-base);
}

.photo-card:hover {
  border-color: var(--android-blue);
  box-shadow: var(--shadow-md);
}

.photo-card.selected {
  border-color: var(--android-blue);
  box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.2);
}

.photo-card img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.photo-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity var(--transition-base);
}

.photo-card:hover .photo-overlay {
  opacity: 1;
}

.photo-number {
  color: white;
  font-size: 24px;
  font-weight: 700;
}

.delete-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 28px;
  height: 28px;
  background: rgba(255, 59, 48, 0.9);
  color: white;
  border: none;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background var(--transition-base);
}

.delete-btn:hover {
  background: rgba(255, 59, 48, 1);
}

.delete-btn svg {
  width: 14px;
  height: 14px;
}

.check-mark {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 24px;
  height: 24px;
  background: var(--android-blue);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.check-mark svg {
  width: 14px;
  height: 14px;
}

.selection-footer {
  padding: 16px 24px;
  background: white;
  border-top: 1px solid var(--border-color);
}

.confirm-btn {
  width: 100%;
  padding: 14px;
  font-size: 16px;
  min-height: 48px;
}

@media (max-width: 768px) {
  .selection-content {
    padding: 16px;
  }

  .template-grid {
    flex-direction: column;
  }

  .photo-grid {
    grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
  }
}
</style>
