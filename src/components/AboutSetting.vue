<script setup>
import { Info, Mail, MessageCircle, ExternalLink, Heart, Code, Zap } from 'lucide-vue-next'
import Button from './ui/Button.vue'

const version = '1.0.0'
const buildDate = '2025-02-16'

const contactInfo = {
  email: 'support@linkron.app',
  wechat: 'linkron_official',
  qqGroup: '123456789',
  github: 'https://github.com/ittking/linkron',
  website: 'https://linkron.app'
}

const features = [
  {
    icon: Zap,
    title: '高效笔记',
    description: '快速创建、管理和搜索您的笔记'
  },
  {
    icon: MessageCircle,
    title: 'AI 助手',
    description: '智能优化内容，提升笔记质量'
  },
  {
    icon: Code,
    title: '标签系统',
    description: '灵活的标签分类，快速定位笔记'
  }
]
</script>

<template>
  <div class="space-y-4">
    <!-- 版本信息 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <h2 class="card-title text-sm font-medium mb-4">
          <Info :size="16" />
          关于 LINKRON
        </h2>

        <div class="flex items-center gap-4 mb-4">
          <div class="w-16 h-16 rounded-xl bg-gradient-to-br from-primary to-secondary flex items-center justify-center">
            <span class="text-2xl font-bold text-base-content">L</span>
          </div>
          <div>
            <h3 class="text-lg font-semibold">LINKRON</h3>
            <p class="text-sm text-base-content/60">智能笔记助手</p>
            <div class="flex items-center gap-2 mt-1">
              <span class="text-xs text-base-content/40">版本 {{ version }}</span>
              <span class="text-xs text-base-content/30">|</span>
              <span class="text-xs text-base-content/40">{{ buildDate }}</span>
            </div>
          </div>
        </div>

        <p class="text-xs text-base-content/60 leading-relaxed">
          LINKRON 是一款基于 Tauri + Vue 3 构建的跨平台桌面应用，旨在提供高效、智能的笔记管理体验。
        </p>
      </div>
    </div>

    <!-- 核心功能 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <h2 class="card-title text-sm font-medium mb-3">
          <Zap :size="16" />
          核心功能
        </h2>
        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
          <div
            v-for="feature in features"
            :key="feature.title"
            class="p-3 rounded-lg bg-base-100 border border-base-300"
          >
            <div class="flex items-center gap-2 mb-2">
              <component :is="feature.icon" :size="14" class="text-primary" />
              <span class="text-sm font-medium">{{ feature.title }}</span>
            </div>
            <p class="text-xs text-base-content/60">{{ feature.description }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 联系方式 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <h2 class="card-title text-sm font-medium mb-3">
          <Mail :size="16" />
          联系我们
        </h2>
        <div class="space-y-2">
          <div class="flex items-center justify-between p-2 rounded-lg bg-base-100">
            <div class="flex items-center gap-2">
              <Mail :size="14" class="text-base-content/40" />
              <span class="text-sm">官方邮箱</span>
            </div>
            <a
              :href="`mailto:${contactInfo.email}`"
              class="text-xs text-primary hover:text-primary/80 transition-colors"
            >
              {{ contactInfo.email }}
            </a>
          </div>

          <div class="flex items-center justify-between p-2 rounded-lg bg-base-100">
            <div class="flex items-center gap-2">
              <MessageCircle :size="14" class="text-base-content/40" />
              <span class="text-sm">微信公众号</span>
            </div>
            <span class="text-xs text-base-content/60">{{ contactInfo.wechat }}</span>
          </div>

          <div class="flex items-center justify-between p-2 rounded-lg bg-base-100">
            <div class="flex items-center gap-2">
              <MessageCircle :size="14" class="text-base-content/40" />
              <span class="text-sm">QQ 群</span>
            </div>
            <span class="text-xs text-base-content/60">{{ contactInfo.qqGroup }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 相关链接 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <h2 class="card-title text-sm font-medium mb-3">
          <ExternalLink :size="16" />
          相关链接
        </h2>
        <div class="space-y-2">
          <a
            :href="contactInfo.website"
            target="_blank"
            rel="noopener noreferrer"
            class="flex items-center justify-between p-2 rounded-lg bg-base-100 hover:bg-base-200 transition-colors group"
          >
            <span class="text-sm">官方网站</span>
            <ExternalLink :size="12" class="text-base-content/40 group-hover:text-primary transition-colors" />
          </a>

          <a
            :href="contactInfo.github"
            target="_blank"
            rel="noopener noreferrer"
            class="flex items-center justify-between p-2 rounded-lg bg-base-100 hover:bg-base-200 transition-colors group"
          >
            <span class="text-sm">GitHub 仓库</span>
            <ExternalLink :size="12" class="text-base-content/40 group-hover:text-primary transition-colors" />
          </a>
        </div>
      </div>
    </div>

    <!-- 致谢 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <h2 class="card-title text-sm font-medium mb-2">
          <Heart :size="16" />
          致谢
        </h2>
        <p class="text-xs text-base-content/60 leading-relaxed">
          感谢所有为 LINKRON 做出贡献的开发者和用户。特别感谢以下开源项目：
        </p>
        <div class="flex flex-wrap gap-2 mt-2">
          <span class="badge badge-xs badge-ghost">Tauri</span>
          <span class="badge badge-xs badge-ghost">Vue 3</span>
          <span class="badge badge-xs badge-ghost">DaisyUI</span>
          <span class="badge badge-xs badge-ghost">TipTap</span>
          <span class="badge badge-xs badge-ghost">SQLite</span>
        </div>
      </div>
    </div>
  </div>
</template>
