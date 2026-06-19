import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

export type CaptureMode = 'grid4' | 'newspaper' | 'live';
export type DeviceOrientation = 'portrait' | 'landscape';
export type FilterType = 'none' | 'bw' | 'sepia' | 'warm' | 'cool' | 'vivid';
export type TemplateLayout = 'grid4' | 'single' | 'strip';

export interface PhotoItem {
  dataUrl: string;
  filePath?: string;
  isLive?: boolean;
}

/** 滤镜 → CSS filter 字符串 (用于预览和 canvas 渲染) */
export const FILTER_CSS: Record<FilterType, string> = {
  none: 'none',
  bw: 'grayscale(1) contrast(1.05)',
  sepia: 'sepia(0.7) contrast(1.1) brightness(1.05)',
  warm: 'saturate(1.3) hue-rotate(-10deg) brightness(1.05)',
  cool: 'saturate(1.2) hue-rotate(15deg) brightness(0.98)',
  vivid: 'saturate(1.6) contrast(1.15)',
};

export const FILTER_LABELS: Record<FilterType, string> = {
  none: '原图',
  bw: '黑白',
  sepia: '复古',
  warm: '暖色',
  cool: '冷色',
  vivid: '鲜艳',
};

export const TEMPLATES: { id: TemplateLayout; name: string; cols: number; rows: number; slots: number }[] = [
  { id: 'grid4', name: '四宫格', cols: 2, rows: 2, slots: 4 },
  { id: 'single', name: '单张', cols: 1, rows: 1, slots: 1 },
  { id: 'strip', name: '长条', cols: 1, rows: 4, slots: 4 },
];

export const useCaptureStore = defineStore('capture', () => {
  const mode = ref<CaptureMode>('grid4');
  const photos = ref<PhotoItem[]>([]);
  const selectedIndex = ref<number>(-1);
  const deviceConnected = ref(false);
  const deviceOrientation = ref<DeviceOrientation>('portrait');
  const templateId = ref<string>('grid4');
  const filter = ref<FilterType>('none');

  // 选片结果
  const selectedPhotoIndices = ref<number[]>([]);

  // 打印设置
  const paperSize = ref('4x6');
  const printMode = ref('color');
  const copies = ref(1);

  const maxPhotos = computed(() => {
    const tpl = TEMPLATES.find((t) => t.id === templateId.value);
    return tpl ? tpl.slots : 4;
  });

  function setMode(m: CaptureMode) {
    mode.value = m;
    photos.value = [];
    selectedIndex.value = -1;
  }

  function setTemplate(id: string) {
    templateId.value = id;
    // 切换模板时清空超出数量的照片
    const tpl = TEMPLATES.find((t) => t.id === id);
    if (tpl && photos.value.length > tpl.slots) {
      photos.value = photos.value.slice(0, tpl.slots);
    }
  }

  function setFilter(f: FilterType) {
    filter.value = f;
  }

  function addPhoto(photo: PhotoItem) {
    if (photos.value.length < maxPhotos.value) {
      photos.value.push(photo);
      selectedIndex.value = photos.value.length - 1;
    }
  }

  function selectPhoto(index: number) {
    selectedIndex.value = index;
  }

  function removePhoto(index: number) {
    photos.value.splice(index, 1);
    if (selectedIndex.value >= photos.value.length) {
      selectedIndex.value = Math.max(0, photos.value.length - 1);
    }
  }

  function clearPhotos() {
    photos.value = [];
    selectedIndex.value = -1;
    selectedPhotoIndices.value = [];
  }

  function setSelectedPhotos(indices: number[]) {
    selectedPhotoIndices.value = [...indices];
  }

  /** 获取要用于排版/打印的照片（按选片顺序，未选则全部） */
  const photosForLayout = computed(() => {
    if (selectedPhotoIndices.value.length > 0) {
      return selectedPhotoIndices.value
        .map((i) => photos.value[i])
        .filter((p): p is PhotoItem => !!p);
    }
    return photos.value;
  });

  return {
    mode, photos, selectedIndex, deviceConnected,
    deviceOrientation, templateId, filter, maxPhotos,
    selectedPhotoIndices, paperSize, printMode, copies,
    photosForLayout,
    setMode, setTemplate, setFilter, addPhoto, selectPhoto,
    removePhoto, clearPhotos, setSelectedPhotos,
  };
});
