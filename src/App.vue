<template>
  <div class="h-screen flex flex-col bg-bg-primary">
    <header class="flex items-center justify-between px-6 py-3 bg-white border-b border-border-color shadow-sm-custom z-100">
      <div class="flex items-center gap-3">
        <CameraOutlined class="text-primary" style="font-size: 24px;" />
        <h1 class="text-xl font-bold text-text-primary m-0">大头贴</h1>
      </div>
      <a-tabs v-model:active-key="currentView" size="small">
        <a-tab-pane key="capture" tab="拍照" />
        <a-tab-pane key="select" tab="选片" />
        <a-tab-pane key="print" tab="打印" />
      </a-tabs>
      <a-button type="text" @click="showSettings = true" class="!p-2 !rounded-full hover:bg-bg-tertiary">
        <template #icon><SettingOutlined /></template>
      </a-button>
    </header>
    <main class="flex-1 overflow-hidden relative" role="main">
      <CaptureView v-if="currentView === 'capture'" @complete="currentView = 'select'" />
      <SelectView v-if="currentView === 'select'" @confirm="currentView = 'print'" @back="currentView = 'capture'" />
      <PrintView v-if="currentView === 'print'" @back="currentView = 'select'" />
    </main>
    <Teleport to="body">
      <SettingsModal :visible="showSettings" @close="showSettings = false" />
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { SettingOutlined, CameraOutlined } from '@ant-design/icons-vue';
import CaptureView from './views/CaptureView.vue';
import SelectView from './views/SelectView.vue';
import PrintView from './views/PrintView.vue';
import SettingsModal from './components/SettingsModal.vue';
import { useCameraStore } from './stores/camera';

type ViewName = 'capture' | 'select' | 'print';
const currentView = ref<ViewName>('capture');
const showSettings = ref(false);
const cameraStore = useCameraStore();

// 打开 app 后自动连接 iPhone
onMounted(() => {
  cameraStore.connect();
});
</script>
