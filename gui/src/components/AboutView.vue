<script setup lang="ts">
import { computed } from 'vue';
import {
  Search,
  Sparkles,
  ShieldCheck,
  Heart,
  Github,
  ExternalLink,
} from 'lucide-vue-next';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useI18n } from '../composables/useI18n';

const { t } = useI18n();

const useCases = computed(() => [
  {
    icon: Search,
    title: t('about.useCase1Title'),
    desc: t('about.useCase1Desc'),
  },
  {
    icon: Sparkles,
    title: t('about.useCase2Title'),
    desc: t('about.useCase2Desc'),
  },
  {
    icon: ShieldCheck,
    title: t('about.useCase3Title'),
    desc: t('about.useCase3Desc'),
  },
]);

async function handleOpenGithub() {
  const url = 'https://github.com/huiyang233/disk-diff';
  try {
    await openUrl(url);
  } catch {
    window.open(url, '_blank');
  }
}
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
          <p class="subtitle">{{ t('about.subtitle') }}</p>
        </div>
      </div>

      <div class="divider-line" />

      <!-- Origin Story -->
      <div class="story-section">
        <div class="section-title-row">
          <Heart :size="15" class="story-icon" />
          <h3>{{ t('about.storyTitle') }}</h3>
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
        <h3>{{ t('about.useCasesTitle') }}</h3>
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

      <!-- GitHub Repository Link Section -->
      <div class="github-section">
        <div class="github-row">
          <div class="github-left">
            <Github :size="18" class="github-icon" />
            <div class="github-text">
              <span class="github-title">{{ t('about.githubRepo') }}</span>
              <a
                class="github-url"
                href="https://github.com/huiyang233/disk-diff"
                target="_blank"
                @click.prevent="handleOpenGithub"
              >
                https://github.com/huiyang233/disk-diff
              </a>
            </div>
          </div>
          <button class="btn-secondary btn-sm github-btn" @click="handleOpenGithub">
            <ExternalLink :size="13" />
            <span>{{ t('about.openGithub') }}</span>
          </button>
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

.github-section {
  padding: 10px 14px;
  background: rgba(0, 0, 0, 0.25);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
}

.github-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.github-left {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.github-icon {
  color: var(--text-primary);
  flex-shrink: 0;
}

.github-text {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.github-title {
  font-size: 11px;
  color: var(--text-muted);
  font-weight: 500;
}

.github-url {
  font-size: 12px;
  color: var(--accent-cyan);
  font-family: var(--font-mono);
  text-decoration: none;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.github-url:hover {
  text-decoration: underline;
}

.github-btn {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}
</style>
