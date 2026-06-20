<template>
  <div class="flex flex-col h-full gap-4 p-4">
    <!-- 预览区域 -->
    <div class="flex-1 flex flex-col bg-white rounded-lg border border-border-color overflow-hidden min-h-[360px]">
      <!-- 未连接：引导连接 -->
      <div
        v-if="!isCameraRunning"
        class="flex-1 flex flex-col items-center justify-center text-center p-8"
      >
        <CameraOutlined class="text-5xl text-text-tertiary mb-4" />
        <p class="text-lg font-medium text-text-primary mb-2">连接 iPhone</p>
        <p class="text-sm text-text-secondary mb-6">通过 USB 连接 iPhone 作为外接相机</p>
        <a-button
          v-if="!deviceId && !showFallbackOption"
          type="primary"
          size="large"
          :loading="connecting"
          @click="startCamera"
        >
          <template #icon><ScanOutlined /></template>
          连接手机
        </a-button>
        <div v-if="!deviceId && showFallbackOption" class="flex flex-col items-center gap-3">
          <p class="text-sm text-text-secondary mb-1">未检测到 iPhone 设备</p>
          <a-button
            type="primary"
            size="large"
            :loading="connecting"
            @click="startBuiltInCamera"
          >
            <template #icon><CameraOutlined /></template>
            使用 Mac 内置摄像头
          </a-button>
          <a-button
            v-if="hasLibMobileDevice"
            type="default"
            size="large"
            @click="showFallbackOption = false; startCamera()"
          >
            我有 iPhone，重新检测
          </a-button>
        </div>
        <div v-else-if="deviceId" class="flex flex-col items-center gap-3">
          <a-tag color="blue">设备: {{ deviceId.slice(0, 8) }}…</a-tag>
          <a-button type="primary" size="large" :loading="connecting" @click="startCamera">
            <template #icon><ScanOutlined /></template>
            重新连接
          </a-button>
        </div>
      </div>

      <!-- 已连接：MJPEG 预览 -->
      <div v-else class="relative flex-1 flex items-center justify-center bg-black">
        <img
          :src="mjpegUrl"
          class="w-full h-full object-contain"
          :style="{ filter: previewFilter }"
          alt="相机预览"
        />
        <a-badge
          :status="cameraStore.state.useBuiltInCamera ? 'warning' : 'processing'"
          :text="cameraStore.state.useBuiltInCamera ? '内置摄像头' : '预览中'"
          class="absolute top-3 right-3"
        />
      </div>
    </div>

    <!-- 控制栏：模板 + 滤镜 -->
    <div class="flex flex-wrap items-center gap-4 px-2">
      <div class="flex items-center gap-2">
        <span class="text-sm font-medium text-text-secondary">模板</span>
        <a-radio-group
          :value="captureStore.layoutId"
          button-style="solid"
          size="small"
          @change="(e: any) => captureStore.setLayout(e.target.value)"
        >
          <a-radio-button v-for="l in layouts" :key="l.id" :value="l.id">
            {{ l.name }}
          </a-radio-button>
        </a-radio-group>
      </div>

      <div class="flex items-center gap-2">
        <span class="text-sm font-medium text-text-secondary">滤镜</span>
        <a-radio-group
          :value="captureStore.filter"
          button-style="solid"
          size="small"
          @change="(e: any) => captureStore.setFilter(e.target.value)"
        >
          <a-radio-button v-for="f in filterKeys" :key="f" :value="f">
            {{ filterLabels[f] }}
          </a-radio-button>
        </a-radio-group>
      </div>
    </div>

    <!-- 拍照按钮 -->
    <div class="text-center">
      <a-button
        type="primary"
        size="large"
        shape="round"
        :disabled="!isCameraRunning || captureStore.photos.length >= captureStore.maxPhotos"
        :loading="capturing"
        class="!bg-[#34C759] !border-[#34C759] hover:!bg-[#2DB84E] hover:!border-[#2DB84E] !px-10 !h-12"
        @click="takePhoto"
      >
        <template #icon><CameraOutlined /></template>
        {{ capturing ? '拍摄中...' : '拍照' }}
        <span class="ml-2 opacity-70">({{ captureStore.photos.length }}/{{ captureStore.maxPhotos }})</span>
      </a-button>
    </div>

    <!-- 已拍照片缩略图条 -->
    <div
      v-if="captureStore.photos.length > 0"
      class="flex items-center gap-2 overflow-x-auto pb-1"
    >
      <div
        v-for="(photo, idx) in captureStore.photos"
        :key="idx"
        class="relative w-20 h-20 rounded-lg overflow-hidden flex-shrink-0 border border-border-color group"
      >
        <img :src="photo.dataUrl" class="w-full h-full object-cover" alt="已拍照片" />
        <button
          class="absolute top-1 right-1 w-5 h-5 bg-black/60 text-white rounded-full flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity hover:bg-danger"
          @click="removePhoto(idx)"
          aria-label="删除照片"
        >
          <CloseOutlined style="font-size: 11px;" />
        </button>
        <span class="absolute bottom-1 left-1 px-1.5 py-0.5 bg-black/60 text-white text-[10px] rounded">
          {{ idx + 1 }}
        </span>
      </div>

      <!-- 下一步按钮 -->
      <a-button
        type="primary"
        size="large"
        class="flex-shrink-0 ml-auto"
        @click="$emit('complete')"
      >
        下一步
        <template #icon><ArrowRightOutlined /></template>
      </a-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  CameraOutlined,
  ScanOutlined,
  CloseOutlined,
  ArrowRightOutlined,
} from '@ant-design/icons-vue';
import { ref, computed, onUnmounted, h } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { message, Modal } from 'ant-design-vue';
import {
  useCaptureStore,
  FILTER_CSS,
  FILTER_LABELS,
  LAYOUTS,
  type FilterType,
} from '../stores/capture';
import { useCameraStore } from '../stores/camera';

defineEmits<{
  complete: [];
}>();

const captureStore = useCaptureStore();
const cameraStore = useCameraStore();

const deviceId = ref('');
const connecting = ref(false);
const capturing = ref(false);
const showFallbackOption = ref(false);
const hasLibMobileDevice = ref(true);
const mjpegUrl = 'http://127.0.0.1:27183';

const layouts = LAYOUTS;
const filterKeys = Object.keys(FILTER_LABELS) as FilterType[];
const filterLabels = FILTER_LABELS;

const isCameraRunning = computed(() => cameraStore.isConnected);
const previewFilter = computed(() => FILTER_CSS[captureStore.filter]);

/** 设备信息（用于选择弹窗） */
interface DeviceEntry {
  id: string;
  name: string;
  shortId: string;
}

/** 连接 iPhone 相机 */
async function startCamera() {
  connecting.value = true;
  message.loading({ content: '正在连接 iPhone...', key: 'camera', duration: 0 });
  try {
    console.log('[CaptureView] startCamera: 开始枚举设备');
    const devicesRaw = await invoke<string[]>('discover_devices');
    const devices: string[] = devicesRaw;
    console.log(`[CaptureView] discover_devices: 找到 ${devices.length} 台设备`, devices);

    // 2) 没有 iPhone → 提示内置摄像头
    if (devices.length === 0) {
      console.log('[CaptureView] startCamera: 未检测到 iPhone，显示内置摄像头选项');
      showFallbackOption.value = true;
      message.destroy('camera');
      return;
    }

    // 3) 多个 iPhone → 弹出选择弹窗
    if (devices.length > 1) {
      console.log('[CaptureView] startCamera: 检测到多台 iPhone，弹出选择');
      await showDeviceSelector(devices);
      connecting.value = false;
      return;
    }

    // 4) 单台 iPhone → 直接连接
    deviceId.value = devices[0];
    console.log(`[CaptureView] startCamera: 准备连接设备 ${devices[0].slice(0, 8)}...`);
    const result = await invoke('start_camera', { deviceId: devices[0] });
    console.log(`[CaptureView] start_camera 返回:`, result);
    cameraStore.setState({
      connected: true,
      running: true,
      deviceId: devices[0],
      deviceName: '',
      useBuiltInCamera: false,
    });
    message.success({ content: 'iPhone 已连接', key: 'camera', duration: 2 });
  } catch (e) {
    message.destroy('camera');
    const msg = e instanceof Error ? e.message : String(e);
    console.error('[CaptureView] startCamera 失败:', msg);
    // 如果是 "未找到 idevice_id" 错误，说明 libimobiledevice 未安装
    if (msg.includes('未找到') && msg.includes('idevice_id')) {
      hasLibMobileDevice.value = false;
      showFallbackOption.value = true;
    }
    // 如果错误来自 Swift helper（包含 "未检测到 iPhone 相机" 或 "相机启动失败"），
    // 直接显示错误消息，不再跑诊断（Swift 已经给出了完整解释）
    else if (msg.includes('未检测到 iPhone 相机') || msg.includes('相机启动失败')) {
      Modal.error({
        title: '连接失败',
        content: msg,
        width: 600,
      });
    } else {
      await showDiagnostics(msg);
    }
  } finally {
    connecting.value = false;
  }
}

/** 弹出设备选择弹窗 */
async function showDeviceSelector(deviceIds: string[]) {
  // 批量获取设备信息
  const entries: DeviceEntry[] = [];
  for (const id of deviceIds) {
    try {
      const info = await invoke<Record<string, any>>('get_device_info', { deviceId: id });
      entries.push({
        id,
        name: info.name as string || `iPhone (${id.slice(0, 8)})`,
        shortId: id.slice(0, 8),
      });
    } catch {
      entries.push({
        id,
        name: `iPhone (${id.slice(0, 8)})`,
        shortId: id.slice(0, 8),
      });
    }
  }

  return new Promise<void>((resolve) => {
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
              resolve();
            },
          }, [
            h('span', { class: 'text-sm' }, e.name),
            h('span', {
              class: 'text-xs text-gray-400 ml-auto',
            }, e.shortId),
          ]),
        ),
      ]),
      closable: true,
      maskClosable: true,
      width: 420,
    });
  });
}

/** 连接到指定设备 */
async function connectToDevice(deviceIdToConnect: string) {
  console.log(`[CaptureView] connectToDevice: ${deviceIdToConnect.slice(0, 8)}`);
  message.loading({ content: `正在连接...`, key: 'camera', duration: 0 });
  try {
    const result = await invoke('start_camera', { deviceId: deviceIdToConnect });
    console.log(`[CaptureView] start_camera 返回:`, result);
    cameraStore.setState({
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
    console.error('[CaptureView] connectToDevice 失败:', msg);
    if (msg.includes('未检测到 iPhone 相机') || msg.includes('相机启动失败')) {
      Modal.error({
        title: '连接失败',
        content: msg,
        width: 600,
      });
    } else {
      await showDiagnostics(msg);
    }
  }
}

/** 使用 Mac 内置摄像头 */
async function startBuiltInCamera() {
  connecting.value = true;
  message.loading({ content: '正在启动内置摄像头...', key: 'camera', duration: 0 });
  try {
    console.log('[CaptureView] startBuiltInCamera: 启动内置摄像头');
    const result = await invoke('start_builtin_camera');
    console.log(`[CaptureView] start_builtin_camera 返回:`, result);
    cameraStore.setState({
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
    console.error('[CaptureView] startBuiltInCamera 失败:', msg);
    if (msg.includes('未检测到 iPhone 相机') || msg.includes('相机启动失败')) {
      Modal.error({
        title: '连接失败',
        content: msg,
        width: 600,
      });
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
    const report = (await invoke('diagnose_connection')) as Record<string, any>;
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

    const devices = report.devices || [];
    if (devices.length === 0) {
      lines.push('✗ 未检测到通过 USB 连接的 iPhone');
      lines.push('  请: 1) 用数据线连接 iPhone; 2) 解锁 iPhone; 3) 点击「信任此电脑」');
    } else {
      lines.push(`✓ 检测到 ${devices.length} 台设备:`);
      devices.forEach((d: any) => {
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

/** 拍照 */
async function takePhoto() {
  capturing.value = true;
  try {
    const devId = cameraStore.state.deviceId;
    // 内置摄像头模式下 deviceId 为空，但仍可拍照
    if (!devId && !cameraStore.state.useBuiltInCamera) {
      message.warning('请先连接设备');
      return;
    }
    const photoPath = (await invoke('capture_photo', { deviceId: devId })) as string;
    const base64 = (await invoke('read_photo', { filename: photoPath })) as string;
    captureStore.addPhoto({
      dataUrl: `data:image/jpeg;base64,${base64}`,
      filePath: photoPath,
    });
    message.success('拍照成功');
  } catch (e) {
    message.error(e instanceof Error ? e.message : String(e));
  } finally {
    capturing.value = false;
  }
}

function removePhoto(index: number) {
  captureStore.removePhoto(index);
}

onUnmounted(async () => {
  try {
    await invoke('stop_camera');
    cameraStore.reset();
  } catch {
    // 清理时忽略错误
  }
});
</script>
