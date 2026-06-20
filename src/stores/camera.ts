import { defineStore } from 'pinia';
import { ref, computed, h } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { message, Modal } from 'ant-design-vue';

/** 相机连接状态 */
export interface CameraState {
  connected: boolean;
  running: boolean;
  deviceId: string;
  deviceName: string;
  /** 是否使用 Mac 内置摄像头（而非 iPhone） */
  useBuiltInCamera: boolean;
}

/** 设备信息（用于选择弹窗） */
interface DeviceEntry {
  id: string;
  name: string;
  shortId: string;
}

/**
 * 相机连接 Store
 * 管理 iPhone 设备的连接状态与连接逻辑，供各视图共享
 */
export const useCameraStore = defineStore('camera', () => {
  const state = ref<CameraState>({
    connected: false,
    running: false,
    deviceId: '',
    deviceName: '',
    useBuiltInCamera: false,
  });

  /** 是否正在连接中 */
  const connecting = ref(false);
  /** 是否显示内置摄像头降级选项（未检测到 iPhone） */
  const showFallbackOption = ref(false);
  /** libimobiledevice 是否可用 */
  const hasLibMobileDevice = ref(true);
  /** 设备选择弹窗是否已打开（防止重复触发） */
  const selectorOpen = ref(false);

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
      useBuiltInCamera: false,
    };
  }

  /**
   * 主入口：自动连接 iPhone
   * - 0 台设备：显示降级选项
   * - 1 台设备：直接连接
   * - 多台设备：弹出选择弹窗
   */
  async function connect() {
    // 已连接，不重复连接
    if (isConnected.value) return;
    // 正在连接中，不重复触发
    if (connecting.value) return;
    // 设备选择弹窗已打开，不重复触发
    if (selectorOpen.value) return;
    // 已显示降级选项，不自动重试（需用户手动触发）
    if (showFallbackOption.value) return;

    connecting.value = true;
    message.loading({ content: '正在连接 iPhone...', key: 'camera', duration: 0 });
    try {
      const devices = await invoke<string[]>('discover_devices');

      if (devices.length === 0) {
        showFallbackOption.value = true;
        message.destroy('camera');
        return;
      }

      if (devices.length > 1) {
        // 多台设备：弹出选择弹窗
        // 释放 connecting 锁，由 connectToDevice 在用户选择后重新获取
        message.destroy('camera');
        connecting.value = false;
        selectorOpen.value = true;
        try {
          await showDeviceSelector(devices);
        } finally {
          selectorOpen.value = false;
        }
        return;
      }

      // 单台 iPhone → 直接连接
      await connectToDevice(devices[0]);
    } catch (e) {
      message.destroy('camera');
      const msg = e instanceof Error ? e.message : String(e);
      if (msg.includes('未找到') && msg.includes('idevice_id')) {
        hasLibMobileDevice.value = false;
        showFallbackOption.value = true;
      } else if (msg.includes('未检测到 iPhone 相机') || msg.includes('相机启动失败')) {
        Modal.error({ title: '连接失败', content: msg, width: 600 });
      } else {
        await showDiagnostics(msg);
      }
    } finally {
      connecting.value = false;
    }
  }

  /** 弹出设备选择弹窗 */
  async function showDeviceSelector(deviceIds: string[]) {
    const entries: DeviceEntry[] = [];
    for (const id of deviceIds) {
      try {
        const info = await invoke<Record<string, unknown>>('get_device_info', { deviceId: id });
        const name = (info.name as string | undefined) || `iPhone (${id.slice(0, 8)})`;
        entries.push({ id, name, shortId: id.slice(0, 8) });
      } catch {
        entries.push({
          id,
          name: `iPhone (${id.slice(0, 8)})`,
          shortId: id.slice(0, 8),
        });
      }
    }

    return new Promise<void>((resolve) => {
      let resolved = false;
      const safeResolve = () => {
        if (!resolved) {
          resolved = true;
          resolve();
        }
      };
      const instance = Modal.confirm({
        title: '检测到多台 iPhone',
        content: h('div', [
          h('p', { class: 'mb-2' }, '请选择要连接的设备：'),
          ...entries.map((e) =>
            h('div', {
              class: 'flex items-center gap-2 py-1.5 cursor-pointer hover:bg-gray-50 rounded px-2 -mx-2',
              onClick: () => {
                instance.destroy();
                connectToDevice(e.id);
                safeResolve();
              },
            }, [
              h('span', { class: 'text-sm' }, e.name),
              h('span', { class: 'text-xs text-gray-400 ml-auto' }, e.shortId),
            ]),
          ),
        ]),
        closable: true,
        maskClosable: true,
        width: 420,
        onCancel: safeResolve,
      });
    });
  }

  /** 连接到指定设备 */
  async function connectToDevice(deviceIdToConnect: string) {
    connecting.value = true;
    message.loading({ content: '正在连接...', key: 'camera', duration: 0 });
    try {
      await invoke('start_camera', { deviceId: deviceIdToConnect });
      setState({
        connected: true,
        running: true,
        deviceId: deviceIdToConnect,
        deviceName: '',
        useBuiltInCamera: false,
      });
      message.success({ content: 'iPhone 已连接', key: 'camera', duration: 2 });
    } catch (e) {
      message.destroy('camera');
      const msg = e instanceof Error ? e.message : String(e);
      if (msg.includes('未检测到 iPhone 相机') || msg.includes('相机启动失败')) {
        Modal.error({ title: '连接失败', content: msg, width: 600 });
      } else {
        await showDiagnostics(msg);
      }
    } finally {
      connecting.value = false;
    }
  }

  /** 使用 Mac 内置摄像头 */
  async function startBuiltInCamera() {
    connecting.value = true;
    message.loading({ content: '正在启动内置摄像头...', key: 'camera', duration: 0 });
    try {
      await invoke('start_builtin_camera');
      setState({
        connected: true,
        running: true,
        deviceId: '',
        deviceName: 'Mac 内置摄像头',
        useBuiltInCamera: true,
      });
      message.success({ content: '内置摄像头已启用', key: 'camera', duration: 2 });
    } catch (e) {
      message.destroy('camera');
      const msg = e instanceof Error ? e.message : String(e);
      if (msg.includes('未检测到 iPhone 相机') || msg.includes('相机启动失败')) {
        Modal.error({ title: '连接失败', content: msg, width: 600 });
      } else {
        await showDiagnostics(msg);
      }
    } finally {
      connecting.value = false;
    }
  }

  /** 调用后端诊断，把环境状态汇总展示给用户 */
  async function showDiagnostics(prefix?: string) {
    try {
      const report = (await invoke('diagnose_connection')) as Record<string, unknown>;
      const lines: string[] = [];
      if (prefix) lines.push(prefix, '');
      lines.push('【连接诊断】', '');

      if (report.idevice_id_installed === false) {
        lines.push('✗ 未安装 libimobiledevice (idevice_id)');
        lines.push('  请运行: brew install libimobiledevice');
      } else {
        lines.push('✓ libimobiledevice 已安装');
      }

      if (report.helper_found === false) {
        lines.push('✗ 未找到相机辅助工具 ios_camera_stream');
        lines.push('  请构建: cd src-tauri/ios_camera_stream && swift build -c release');
        lines.push('  并复制到: src-tauri/resources/ios_camera_stream');
      } else {
        lines.push('✓ 相机辅助工具就绪');
      }

      if (report.continuity_camera_supported === false) {
        lines.push(`✗ macOS ${report.macos_version} 不支持连续互通相机 (需 13.0+)`);
      } else {
        lines.push(`✓ macOS ${report.macos_version} 支持连续互通相机`);
      }

      const devices = (report.devices as Array<Record<string, unknown>>) || [];
      if (devices.length === 0) {
        lines.push('✗ 未检测到通过 USB 连接的 iPhone');
        lines.push('  请: 1) 用数据线连接 iPhone; 2) 解锁 iPhone; 3) 点击「信任此电脑」');
      } else {
        lines.push(`✓ 检测到 ${devices.length} 台设备:`);
        devices.forEach((d) => {
          if (d.paired) {
            lines.push(`  • ${d.id_short} (${d.name}) — 已配对`);
          } else {
            lines.push(`  • ${d.id_short} — 未配对: ${d.error}`);
          }
        });
      }

      if (report.port_in_use === true) {
        lines.push('⚠ 预览端口 27183 被占用，请关闭占用进程或重启应用');
      }

      lines.push('', '如问题仍存在，请检查系统「隐私与安全 > 相机」是否已授权。');
      Modal.error({
        title: '连接诊断',
        content: lines.join('\n'),
        width: 600,
      });
    } catch {
      Modal.error({
        title: '连接失败',
        content: prefix || '连接失败且诊断不可用',
      });
    }
  }

  return {
    state,
    connecting,
    showFallbackOption,
    hasLibMobileDevice,
    isConnected,
    setState,
    reset,
    connect,
    connectToDevice,
    startBuiltInCamera,
    showDiagnostics,
  };
});
