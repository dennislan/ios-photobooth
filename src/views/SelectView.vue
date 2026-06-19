<template>
  <div class="flex flex-col h-full">
    <div class="flex items-center justify-between px-6 py-3 bg-white border-b border-border-color">
      <h2 class="text-lg font-semibold m-0">选择照片</h2>
      <a-button @click="$emit('back')">
        <template #icon><ArrowLeftOutlined /></template>
        返回
      </a-button>
    </div>

    <div class="flex-1 overflow-y-auto p-6">
      <div class="mb-8">
        <h3 class="text-base font-semibold text-text-primary mb-4">选择模板</h3>
        <a-row :gutter="[16, 16]">
          <a-col :span="8" v-for="tpl in templatePreviews" :key="tpl.id">
            <a-card
              hoverable
              class="transition-all cursor-pointer"
              :class="captureStore.layoutId === tpl.id ? '!ring-2 !ring-primary bg-[rgba(0,122,255,0.05)]' : ''"
              @click="captureStore.setLayout(tpl.id)"
            >
              <div class="aspect-square rounded-md flex gap-1 p-2" :style="{ backgroundColor: tpl.bgColor }">
                <div class="flex-1 grid gap-1" :class="tpl.gridClass">
                  <div v-for="n in tpl.previewCount" :key="n" class="bg-black/10 rounded-sm"></div>
                </div>
              </div>
              <span class="block text-center text-sm font-medium mt-2 text-text-primary">{{ tpl.name }}</span>
            </a-card>
          </a-col>
        </a-row>
      </div>

      <div>
        <h3 class="text-base font-semibold text-text-primary mb-4">
          已拍摄照片 ({{ photos.length }})
          <span class="text-text-secondary font-normal" v-if="captureStore.selectedCount > 0">/ 已选 {{ captureStore.selectedCount }} 张</span>
        </h3>
        <a-empty v-if="photos.length === 0" description="还没有拍摄照片" class="!py-12" />
        <a-row :gutter="[16, 16]" v-else>
          <a-col v-for="(photo, idx) in photos" :key="idx" :xs="8" :sm="6" :md="4">
            <div
              class="relative cursor-pointer rounded-lg overflow-hidden border-2 transition-all"
              :class="isSelected(idx) ? 'border-primary ring-2 ring-[rgba(0,122,255,0.2)]' : 'border-transparent'"
              @click="captureStore.toggleSelection(idx)"
              role="checkbox"
              :aria-checked="isSelected(idx)"
            >
              <img :src="photo.dataUrl" class="w-full aspect-square object-cover" alt="照片" />
              <div v-if="isSelected(idx)" class="absolute top-2 right-2 w-6 h-6 bg-primary text-white rounded-full flex items-center justify-center text-xs font-bold">
                {{ selectionOrder(idx) }}
              </div>
              <button
                class="absolute top-2 left-2 w-6 h-6 bg-black/60 text-white rounded-full flex items-center justify-center opacity-0 hover:opacity-100 transition-opacity hover:bg-danger"
                @click.stop="captureStore.removePhoto(idx)"
                aria-label="删除照片"
              >
                <DeleteOutlined style="font-size: 12px;" />
              </button>
            </div>
          </a-col>
        </a-row>
      </div>
    </div>

    <div class="px-6 py-4 bg-white border-t border-border-color">
      <a-button
        type="primary"
        size="large"
        block
        :disabled="captureStore.selectedCount === 0"
        @click="confirmSelection"
        class="!py-3.5 !text-base"
      >
        确认选择 ({{ captureStore.selectedCount }})
        <template #icon><ArrowRightOutlined /></template>
      </a-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ArrowLeftOutlined, ArrowRightOutlined, DeleteOutlined } from '@ant-design/icons-vue';
import { computed } from 'vue';
import { useCaptureStore, type LayoutType } from '../stores/capture';

const emit = defineEmits<{
  confirm: [];
  back: [];
}>();

const captureStore = useCaptureStore();

const photos = computed(() => captureStore.photos);

const templatePreviews: { id: LayoutType; name: string; bgColor: string; gridClass: string; previewCount: number }[] = [
  { id: 'grid4', name: '四宫格', bgColor: '#F0F0F0', gridClass: 'grid grid-cols-2 grid-rows-2', previewCount: 4 },
  { id: 'single', name: '单张', bgColor: '#E8F4FD', gridClass: 'grid grid-cols-1 grid-rows-1', previewCount: 1 },
  { id: 'strip', name: '长条', bgColor: '#FFF4E6', gridClass: 'grid grid-cols-1 grid-rows-4', previewCount: 4 },
];

function isSelected(idx: number): boolean {
  return captureStore.selectedIndices.includes(idx);
}

function selectionOrder(idx: number): number {
  return captureStore.selectedIndices.indexOf(idx) + 1;
}

function confirmSelection() {
  emit('confirm');
}
</script>
