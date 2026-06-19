<template>
  <div class="flex flex-col items-center justify-center h-full text-center p-8">
    <div class="max-w-[400px]">
      <!-- Brand Logo -->
      <div class="w-[120px] h-[120px] mx-auto mb-8">
        <svg
          viewBox="0 0 120 120"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <circle cx="60" cy="60" r="56" fill="#007AFF" />
          <text
            x="60"
            y="72"
            text-anchor="middle"
            fill="white"
            font-size="48"
            font-weight="bold"
            font-family="-apple-system, BlinkMacSystemFont, sans-serif"
          >
            v
          </text>
        </svg>
      </div>

      <h1 class="text-4xl font-bold text-text-primary mb-4">大头贴</h1>
      <p class="text-xl text-text-secondary mb-12 leading-relaxed">
        点击下方按钮开始拍照体验
      </p>

      <a-button
        type="primary"
        size="large"
        @click="$emit('start-capture')"
        class="!px-12 !py-3 !text-lg"
      >
        <template #icon>
          <CameraOutlined />
        </template>
        开始拍照
      </a-button>
    </div>

    <div class="mt-12 text-text-tertiary text-sm">
      <p>连接 iPhone 即可开始体验</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { CameraOutlined } from "@ant-design/icons-vue"

defineEmits<{
  "start-capture": []
}>()
</script>
