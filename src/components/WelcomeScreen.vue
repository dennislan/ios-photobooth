<template>
  <div class="flex flex-col items-center justify-center h-full text-center p-8">
    <div class="max-w-[400px]">
      <div class="w-[120px] h-[120px] mx-auto mb-8 rounded-full bg-primary flex items-center justify-center shadow-lg-custom">
        <CameraOutlined class="text-5xl text-white" />
      </div>
      <h1 class="text-4xl font-bold text-text-primary mb-4">大头贴</h1>
      <p class="text-xl text-text-secondary mb-12 leading-relaxed">连接 iPhone，开始拍照体验</p>
      <a-button type="primary" size="large" shape="round" @click="$emit('start')" class="!px-12 !h-12 !text-lg">
        <template #icon><CameraOutlined /></template>
        开始拍照
      </a-button>
    </div>
    <div class="mt-12 text-text-tertiary text-sm">
      <p>通过 USB 连接 iPhone 即可开始</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { CameraOutlined } from '@ant-design/icons-vue';
defineEmits<{ start: [] }>();
</script>
