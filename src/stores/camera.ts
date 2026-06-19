import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

/** 相机连接状态 */
export interface CameraState {
  connected: boolean;
  running: boolean;
  deviceId: string;
  deviceName: string;
}

/**
 * 相机连接 Store
 * 管理 iPhone 设备的连接状态，供各视图共享
 */
export const useCameraStore = defineStore('camera', () => {
  const state = ref<CameraState>({
    connected: false,
    running: false,
    deviceId: '',
    deviceName: '',
  });

  /** 设备是否已连接且预览流正在运行 */
  const isConnected = computed(
    () => state.value.connected && state.value.running,
  );

  /** 更新连接状态 */
  function setState(patch: Partial<CameraState>) {
    Object.assign(state.value, patch);
  }

  /** 重置为初始状态 */
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
