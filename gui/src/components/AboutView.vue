<script setup lang="ts">
import { computed } from 'vue';
import {
  Search,
  Sparkles,
  ShieldCheck,
  Heart,
} from 'lucide-vue-next';
import { useI18n } from '../composables/useI18n';

const { t, isZh } = useI18n();

const useCases = computed(() => [
  {
    icon: Search,
    title: isZh.value ? '揪出偷偷膨胀的缓存与日志' : 'Identify Bloated Caches & Logs',
    desc: isZh.value
      ? '日常开发中 node_modules、构建缓存、Docker 镜像或应用日志经常莫名其妙变大，拍两个快照比对一下，哪儿暴涨了一目了然。'
      : 'Quickly detect runaway log files, cache directories, node_modules, or container images growing unexpectedly over time.',
  },
  {
    icon: Sparkles,
    title: isZh.value ? '大扫除前后留底比对' : 'Before & After Cleanup Verification',
    desc: isZh.value
      ? '在清理大文件或卸载软件前后各存一份快照，不仅能直观确认到底释放了多少空间，还能防止误删了重要目录。'
      : 'Capture snapshots before and after cleaning disks or uninstalling heavy software to verify exact space reclaimed.',
  },
  {
    icon: ShieldCheck,
    title: isZh.value ? '纯粹本地运行，隐私无忧' : '100% Local & Privacy-Respecting',
    desc: isZh.value
      ? '所有扫描和快照数据 100% 留在自己电脑本地，零多余网络请求，不做多余无用的功能，专注把磁盘对比这一件事做好。'
      : 'All scanning and snapshot files remain strictly on your local disk with zero telemetry, background tracking, or network requests.',
  },
]);
</script>

<template>
  <div class="about-container">
    <div class="about-card glass-panel">
      <!-- Header -->
      <div class="about-header">
        <div class="logo-box">
          <img src="/app-icon.png" alt="DiskDiff" class="about-app-icon" />
        </div>
        <div class="header-info">
          <div class="title-row">
            <h1>DiskDiff</h1>
            <span class="version-tag">v0.1.1</span>
          </div>
          <p class="subtitle">{{ isZh ? '因为自己有磁盘对比需求而写的实用小工具' : 'A high-performance disk space analyzer & snapshot differential tool' }}</p>
        </div>
      </div>

      <div class="divider-line" />

      <!-- Origin Story -->
      <div class="story-section">
        <div class="section-title-row">
          <Heart :size="15" class="story-icon" />
          <h3>{{ isZh ? '为什么写这个软件？' : 'Why was this built?' }}</h3>
        </div>
        <p class="story-p">
          {{ t('about.story1') }}
        </p>
        <p class="story-p">
          {{ t('about.story2') }}
        </p>
        <p class="story-p">
          {{ t('about.story3') }}
        </p>
      </div>

      <div class="divider-line" />

      <!-- Typical Use Cases -->
      <div class="usecase-section">
        <h3>{{ isZh ? '平时主要用来干嘛？' : 'Common Use Cases' }}</h3>
        <div class="usecase-grid">
          <div
            v-for="item in useCases"
            :key="item.title"
            class="usecase-card"
          >
            <div class="usecase-icon-box">
              <component :is="item.icon" :size="16" class="u-icon" />
            </div>
            <div class="usecase-text">
              <h4>{{ item.title }}</h4>
              <p>{{ item.desc }}</p>
            </div>
          </div>
        </div>
      </div>

      <div class="divider-line" />

      <!-- Tech Stack Badges -->
      <div class="tech-section">
        <span class="tech-label">{{ isZh ? '主要技术：' : 'Built with:' }}</span>
        <div class="badges-row">
          <span class="tech-badge">Tauri 2.0</span>
          <span class="tech-badge">Rust (Rayon + Mimalloc)</span>
          <span class="tech-badge">Vue 3 + TypeScript</span>
          <span class="tech-badge">D3.js Treemap</span>
          <span class="tech-badge">Zstd + Bincode</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.about-container {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  width: 100%;
  padding: 24px;
  background: var(--bg-app);
  overflow-y: auto;
}

.about-card {
  width: 100%;
  max-width: 680px;
  padding: 24px 28px;
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  gap: 16px;
  background: var(--bg-card);
  border: 1px solid var(--border-medium);
}

.about-header {
  display: flex;
  align-items: center;
  gap: 14px;
}

.logo-box {
  width: 52px;
  height: 52px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  overflow: hidden;
}

.about-app-icon {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.header-info {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.title-row h1 {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: -0.01em;
}

.version-tag {
  font-size: 11px;
  font-family: var(--font-mono);
  font-weight: 600;
  color: #38bdf8;
  background: rgba(56, 189, 248, 0.12);
  padding: 1px 7px;
  border-radius: var(--radius-full);
}

.subtitle {
  font-size: 12.5px;
  color: var(--text-secondary);
}

.divider-line {
  height: 1px;
  background: var(--border-subtle);
  width: 100%;
}

/* Story Section */
.story-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-title-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.story-icon {
  color: #f87171;
}

.story-section h3 {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text-primary);
}

.story-p {
  font-size: 12.5px;
  color: var(--text-secondary);
  line-height: 1.65;
}

.story-p strong {
  color: var(--text-primary);
}

/* Use cases */
.usecase-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.usecase-section h3 {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text-primary);
}

.usecase-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.usecase-card {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 10px 12px;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
}

.usecase-icon-box {
  width: 28px;
  height: 28px;
  border-radius: var(--radius-xs);
  background: rgba(56, 189, 248, 0.1);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  margin-top: 1px;
}

.u-icon {
  color: #38bdf8;
}

.usecase-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.usecase-text h4 {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-primary);
}

.usecase-text p {
  font-size: 11.5px;
  color: var(--text-muted);
  line-height: 1.5;
}

.tech-section {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 11.5px;
}

.tech-label {
  color: var(--text-muted);
  white-space: nowrap;
}

.badges-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.tech-badge {
  padding: 2px 7px;
  border-radius: var(--radius-xs);
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--border-subtle);
  font-size: 11px;
  font-family: var(--font-mono);
  color: var(--text-secondary);
}
</style>
