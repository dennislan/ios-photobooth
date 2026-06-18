<template>
  <div class="idle-view">
    <div class="idle-content">
      <div class="brand-logo">
        <svg
          viewBox="0 0 120 120"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <circle cx="60" cy="60" r="56" fill="var(--android-blue)" />
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
      <h1 class="title">android大头贴</h1>
      <p class="subtitle">点击下方按钮开始拍照体验</p>
      <button class="btn btn-primary start-btn" @click="$emit('start-capture')">
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          class="btn-icon-svg"
        >
          <path
            d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"
          />
          <circle cx="12" cy="13" r="4" />
        </svg>
        开始拍照
      </button>
    </div>
    <div class="idle-footer">
      <p>连接 android 手机即可开始体验</p>
    </div>
  </div>
</template>

<script setup lang="ts">
defineEmits<{
  "start-capture": []
}>()
</script>

<style scoped>
.idle-view {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  text-align: center;
  padding: var(--space-xl);
}

.idle-content {
  max-width: 400px;
}

.brand-logo {
  width: 120px;
  height: 120px;
  margin: 0 auto var(--space-xl);
}

.title {
  font-size: 36px;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: var(--space-md);
}

.subtitle {
  font-size: 18px;
  color: var(--text-secondary);
  margin-bottom: var(--space-2xl);
  line-height: 1.6;
}

.start-btn {
  padding: 16px 48px;
  font-size: 20px;
  min-height: 56px;
}

.btn-icon-svg {
  width: 24px;
  height: 24px;
}

.idle-footer {
  margin-top: var(--space-2xl);
  color: var(--text-tertiary);
  font-size: 14px;
}

@media (max-width: 768px) {
  .title {
    font-size: 28px;
  }

  .subtitle {
    font-size: 16px;
  }

  .start-btn {
    padding: 14px 36px;
    font-size: 18px;
  }
}
</style>
