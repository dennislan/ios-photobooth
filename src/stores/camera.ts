import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

export interface CameraState {
  connected: boolean;
  running: boolean;
  deviceId: string;
  deviceName: string;
}

export const useCameraStore = defineStore('camera', () => {
  const state = ref<CameraState>({
    connected: false,
    running: false,
    deviceId: '',
    deviceName: '',
  });

  const isConnected = computed(() => state.value.connected && state.value.running);

  function setState(patch: Partial<CameraState>) {
    Object.assign(state.value, patch);
  }

  function reset() {
    state.value = {
      connected: false,
      running: false,
      deviceId: '',
      deviceName: '',
    };
  }

  return { state, isConnected, setState, reset };
});
