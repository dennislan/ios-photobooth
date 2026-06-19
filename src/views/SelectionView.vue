<template>
  <div class="flex flex-col h-full">
    <!-- Header -->
    <div class="flex items-center justify-between px-6 py-3 bg-white border-b border-border-color">
      <h2 class="text-lg font-semibold m-0">选择照片</h2>
      <a-button @click="$emit('back')">
        <template #icon><ArrowLeftOutlined /></template>
        返回
      </a-button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-6">
      <!-- 模板选择 -->
      <div class="mb-8">
        <h3 class="text-base font-semibold text-text-primary mb-4">选择模板</h3>
        <a-row :gutter="[16, 16]">
          <a-col :span="8" v-for="tpl in templates" :key="tpl.id">
            <a-card
              hoverable
              class="transition-all cursor-pointer"
              :class="captureStore.templateId === tpl.id ? '!ring-2 !ring-primary bg-[rgba(0,122,255,0.05)]' : ''"
              @click="captureStore.setTemplate(tpl.id)"
            >
              <div
                class="aspect-square rounded-md flex gap-1 p-1"
                :style="{ backgroundColor: tpl.bgColor }"
              >
                <div
                  class="flex-1 grid gap-1"
                  :class="tpl.gridClass"
                >
                  <div
                    v-for="n in tpl.previewCount"
                    :key="n"
                    class="bg-[rgba(0,0,0,0.1)] rounded-sm"
                  ></div>
                </div>
              </div>
              <span class="block text-center text-sm font-medium mt-2 text-text-primary">{{ tpl.name }}</span>
            </a-card>
          </a-col>
        </a-row>
      </div>

      <!-- 照片网格 -->
      <div>
        <h3 class="text-base font-semibold text-text-primary mb-4">
          已拍摄照片 ({{ photos.length }})
          <span class="text-text-secondary font-normal" v-if="photos.length > 0">
            / 已选 {{ selectedPhotos.length }} 张
          </span>
        </h3>

        <a-empty
          v-if="photos.length === 0"
          description="还没有拍摄照片哦"
          class="!py-12"
        />

        <a-row :gutter="[16, 16]" v-else>
          <a-col
            v-for="(photo, idx) in photos"
            :key="idx"
            :xs="8"
            :sm="6"
            :md="4"
          >
            <a-card
              hoverable
              class="cursor-pointer transition-all relative"
              :class="selectedPhotos.includes(idx) ? '!ring-2 !ring-primary' : ''"
              @click="toggleSelect(idx)"
              :aria-checked="selectedPhotos.includes(idx)"
              :role="'checkbox'"
            >
              <a-image
                :src="photo.dataUrl"
                :preview="{ src: photo.dataUrl }"
                class="w-full aspect-square"
                style="object-fit: cover; border-radius: 8px;"
              />
              <template #actions>
                <a-tooltip title="删除">
                  <DeleteOutlined @click.stop="removePhoto(idx)" class="text-text-secondary hover:text-danger!" />
                </a-tooltip>
              </template>
              <template #extra>
                <a-badge
                  :count="selectedPhotos.includes(idx) ? 1 : 0"
                  :show-zero="false"
                  :style="{ backgroundColor: selectedPhotos.includes(idx) ? '#007AFF' : undefined }"
                />
              </template>
            </a-card>
          </a-col>
        </a-row>
      </div>
    </div>

    <!-- Footer -->
    <div class="px-6 py-4 bg-white border-t border-border-color">
      <a-button
        type="primary"
        size="large"
        block
        :disabled="selectedPhotos.length === 0"
        @click="confirmSelection"
        class="!py-3.5 !text-base"
      >
        确认选择 ({{ selectedPhotos.length }})
        <template #icon>
          <ArrowRightOutlined />
        </template>
      </a-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  ArrowLeftOutlined,
  ArrowRightOutlined,
  DeleteOutlined,
} from "@ant-design/icons-vue"
import { ref, computed } from "vue"
import { useCaptureStore } from "../stores/capture"

const emit = defineEmits<{
  confirm: []
  back: []
}>()

const captureStore = useCaptureStore()
const selectedPhotos = ref<number[]>([])

const photos = computed(() => captureStore.photos)

const templates = [
  {
    id: "grid4",
    name: "四宫格",
    bgColor: "#F0F0F0",
    gridClass: "grid grid-cols-2 grid-rows-2",
    previewCount: 4,
  },
  {
    id: "single",
    name: "单张",
    bgColor: "#E8F4FD",
    gridClass: "grid grid-cols-1 grid-rows-1",
    previewCount: 1,
  },
  {
    id: "strip",
    name: "长条",
    bgColor: "#FFF4E6",
    gridClass: "grid grid-cols-1 grid-rows-4",
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
  selectedPhotos.value = selectedPhotos.value.filter((i) => i !== index).map((i) => (i > index ? i - 1 : i))
}

function confirmSelection() {
  captureStore.setSelectedPhotos(selectedPhotos.value)
  emit("confirm")
}
</script>
