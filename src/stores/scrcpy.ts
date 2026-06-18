import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

export interface ScrcpyState {
  connected: boolean;
  running: boolean;
  fps: number;
  bitRate: number;
  maxSize: number;
  deviceId: string;
}

export const useScrcpyStore = defineStore('scrcpy', () => {
  const state = ref<ScrcpyState>({
    connected: false,
    running: false,
    fps: 60,
    bitRate: 8000000,
    maxSize: 1920,
    deviceId: '',
  });

  const isConnected = computed(() => state.value.connected && state.value.running);

  function setState(patch: Partial<ScrcpyState>) {
    Object.assign(state.value, patch);
  }

  return { state, isConnected, setState };
});
