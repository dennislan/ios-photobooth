import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

// ── 类型定义 ──

/** 滤镜类型（参照 macOS Photo Booth 的视觉效果） */
export type FilterType = 'none' | 'bw' | 'sepia' | 'warm' | 'cool' | 'vivid';

/** 模板布局类型 */
export type LayoutType = 'grid4' | 'single' | 'strip';

/** 已拍摄照片 */
export interface PhotoItem {
  /** Base64 data URL，供前端显示 */
  dataUrl: string;
  /** 后端文件路径，供打印使用 */
  filePath: string;
}

// ── 常量 ──

/** 滤镜 → CSS filter 字符串（用于预览和 Canvas 渲染） */
export const FILTER_CSS: Record<FilterType, string> = {
  none: 'none',
  bw: 'grayscale(1) contrast(1.05)',
  sepia: 'sepia(0.7) contrast(1.1) brightness(1.05)',
  warm: 'saturate(1.3) hue-rotate(-10deg) brightness(1.05)',
  cool: 'saturate(1.2) hue-rotate(15deg) brightness(0.98)',
  vivid: 'saturate(1.6) contrast(1.15)',
};

/** 滤镜显示标签 */
export const FILTER_LABELS: Record<FilterType, string> = {
  none: '原图',
  bw: '黑白',
  sepia: '复古',
  warm: '暖色',
  cool: '冷色',
  vivid: '鲜艳',
};

/** 模板布局配置 */
export interface LayoutConfig {
  id: LayoutType;
  name: string;
  cols: number;
  rows: number;
  slots: number;
}

export const LAYOUTS: LayoutConfig[] = [
  { id: 'grid4', name: '四宫格', cols: 2, rows: 2, slots: 4 },
  { id: 'single', name: '单张', cols: 1, rows: 1, slots: 1 },
  { id: 'strip', name: '长条', cols: 1, rows: 4, slots: 4 },
];

/** 打印纸张尺寸选项 */
export const PAPER_SIZES = [
  { value: '4x6', label: '4×6 英寸' },
  { value: '5x7', label: '5×7 英寸' },
  { value: '6x8', label: '6×8 英寸' },
  { value: 'A4', label: 'A4' },
];

/** 打印色彩模式选项 */
export const COLOR_MODES = [
  { value: 'color', label: '彩色' },
  { value: 'bw', label: '黑白' },
];

// ── Store ──

/**
 * 拍摄 Store
 * 管理照片列表、模板布局、滤镜、选片结果、打印设置
 */
export const useCaptureStore = defineStore('capture', () => {
  const photos = ref<PhotoItem[]>([]);
  const layoutId = ref<LayoutType>('grid4');
  const filter = ref<FilterType>('none');

  /** 选片结果（照片在 photos 数组中的索引） */
  const selectedIndices = ref<number[]>([]);

  // 打印设置
  const paperSize = ref('4x6');
  const colorMode = ref('color');
  const copies = ref(1);

  /** 当前模板布局配置 */
  const currentLayout = computed(
    () => LAYOUTS.find((t) => t.id === layoutId.value) ?? LAYOUTS[0],
  );

  /** 当前布局最大照片数 */
  const maxPhotos = computed(() => currentLayout.value.slots);

  /** 已选照片数量 */
  const selectedCount = computed(() => selectedIndices.value.length);

  /** 获取用于排版/打印的照片（按选片顺序，未选则全部） */
  const photosForLayout = computed<PhotoItem[]>(() => {
    if (selectedIndices.value.length > 0) {
      return selectedIndices.value
        .map((i) => photos.value[i])
        .filter((p): p is PhotoItem => !!p);
    }
    return photos.value;
  });

  // ── 动作 ──

  function setLayout(id: LayoutType) {
    layoutId.value = id;
    // 切换布局时清空超出数量的照片
    const layout = LAYOUTS.find((t) => t.id === id);
    if (layout && photos.value.length > layout.slots) {
      photos.value = photos.value.slice(0, layout.slots);
    }
  }

  function setFilter(f: FilterType) {
    filter.value = f;
  }

  function addPhoto(photo: PhotoItem) {
    if (photos.value.length < maxPhotos.value) {
      photos.value.push(photo);
    }
  }

  function removePhoto(index: number) {
    photos.value.splice(index, 1);
    // 同步更新选片索引
    selectedIndices.value = selectedIndices.value
      .filter((i) => i !== index)
      .map((i) => (i > index ? i - 1 : i));
  }

  function toggleSelection(index: number) {
    const pos = selectedIndices.value.indexOf(index);
    if (pos >= 0) {
      selectedIndices.value.splice(pos, 1);
    } else {
      selectedIndices.value.push(index);
    }
  }

  function clearPhotos() {
    photos.value = [];
    selectedIndices.value = [];
  }

  /** 重置全部状态（回到初始） */
  function resetAll() {
    clearPhotos();
    layoutId.value = 'grid4';
    filter.value = 'none';
    paperSize.value = '4x6';
    colorMode.value = 'color';
    copies.value = 1;
  }

  return {
    // 状态
    photos,
    layoutId,
    filter,
    selectedIndices,
    paperSize,
    colorMode,
    copies,
    // 计算属性
    currentLayout,
    maxPhotos,
    selectedCount,
    photosForLayout,
    // 动作
    setLayout,
    setFilter,
    addPhoto,
    removePhoto,
    toggleSelection,
    clearPhotos,
    resetAll,
  };
});
