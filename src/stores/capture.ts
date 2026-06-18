import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

export type CaptureMode = 'grid4' | 'newspaper' | 'live';
export type DeviceOrientation = 'portrait' | 'landscape';

export interface PhotoItem {
  dataUrl: string;
  filePath?: string;
  isLive?: boolean;
}

export const useCaptureStore = defineStore('capture', () => {
  const mode = ref<CaptureMode>('grid4');
  const photos = ref<PhotoItem[]>([]);
  const selectedIndex = ref<number>(-1);
  const deviceConnected = ref(false);
  const deviceOrientation = ref<DeviceOrientation>('portrait');
  const templateId = ref<string>('');

  const maxPhotos = computed(() => {
    switch (mode.value) {
      case 'grid4': return 4;
      case 'newspaper': return 1;
      case 'live': return 1;
      default: return 4;
    }
  });

  function setMode(m: CaptureMode) {
    mode.value = m;
    photos.value = [];
    selectedIndex.value = -1;
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
  }

  return {
    mode, photos, selectedIndex, deviceConnected,
    deviceOrientation, templateId, maxPhotos,
    setMode, addPhoto, selectPhoto, removePhoto, clearPhotos,
  };
});
