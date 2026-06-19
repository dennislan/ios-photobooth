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
          v-if="!deviceId"
          type="primary"
          size="large"
          :loading="connecting"
          @click="startCamera"
        >
          <template #icon><ScanOutlined /></template>
          连接手机
        </a-button>
        <div v-else class="flex flex-col items-center gap-3">
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
          status="processing"
          text="预览中"
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
import { ref, computed, onUnmounted } from 'vue';
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
const mjpegUrl = 'http://127.0.0.1:27183';

const layouts = LAYOUTS;
const filterKeys = Object.keys(FILTER_LABELS) as FilterType[];
const filterLabels = FILTER_LABELS;

const isCameraRunning = computed(() => cameraStore.isConnected);
const previewFilter = computed(() => FILTER_CSS[captureStore.filter]);

/** 连接 iPhone 相机 */
async function startCamera() {
  connecting.value = true;
  message.loading({ content: '正在连接 iPhone...', key: 'camera', duration: 0 });
  try {
    const devices = (await invoke('discover_devices')) as string[];
    if (devices.length === 0) {
      message.destroy('camera');
      await showDiagnostics();
      return;
    }
    deviceId.value = devices[0];
    await invoke('start_camera', { deviceId: devices[0] });
    cameraStore.setState({
      connected: true,
      running: true,
      deviceId: devices[0],
    });
    message.success({ content: 'iPhone 已连接', key: 'camera', duration: 2 });
  } catch (e) {
    message.destroy('camera');
    const msg = e instanceof Error ? e.message : String(e);
    await showDiagnostics(msg);
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
    if (!devId) {
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
