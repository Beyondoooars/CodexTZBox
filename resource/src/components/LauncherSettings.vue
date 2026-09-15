<script setup lang="ts">
import { computed } from 'vue'
import { MacButton, MacLabel, MacSegment, MacSegmentedControl, MacTextField } from '@macvue/core'
import type { Settings, Zone } from '../types'
import { offsetText } from '../composables/useClocks'
import AppIcon from './ui/AppIcon.vue'
import AppSelect, { type AppSelectItem } from './ui/AppSelect.vue'
import SettingRow from './ui/SettingRow.vue'

const props = defineProps<{
  settings: Settings
  zones: Zone[]
  detected: string
  platform: string
  busy: boolean
  initializing: boolean
  bootstrapFailed: boolean
  zoneError: string
  currentOffset: string
  previewMode: boolean
}>()

const emit = defineEmits<{
  'update:settings': [settings: Settings]
  browse: []
  retry: []
}>()

const offsets = Array.from({ length: 27 }, (_, index) => index - 12)
const zoneItems = computed<AppSelectItem[]>(() => props.zones.map((zone) => ({
  value: zone.id,
  label: zone.label === zone.id ? zone.id : zone.label,
  detail: zone.label === zone.id ? undefined : zone.id,
})))
const offsetItems = computed<AppSelectItem[]>(() => offsets.map((offset) => ({ value: offset, label: offsetText(offset * 60) })))
const selectedZone = computed(() => props.zones.find((zone) => zone.id === props.settings.zoneId))
const selectedZoneLabel = computed(() => {
  const selected = selectedZone.value
  if (!selected || selected.label === selected.id) return props.settings.zoneId
  return `${selected.label} · ${selected.id}`
})
const previewMenu = import.meta.env.DEV ? new URLSearchParams(window.location.search).get('ui-menu') : null

function update(patch: Partial<Settings>) {
  emit('update:settings', { ...props.settings, ...patch })
}

function setMode(value: string | string[]) {
  if (value === 'zone' || value === 'offset') update({ mode: value })
}

function setZone(value: string | number) {
  if (typeof value === 'string') update({ zoneId: value })
}

function setOffset(value: string | number) {
  if (typeof value === 'number') update({ offset: value })
}
</script>

<template>
  <section class="surface settings-panel" aria-labelledby="settings-title">
    <div v-if="bootstrapFailed" class="inline-alert" role="alert">
      <AppIcon name="alert" />
      <span>设置读取失败，当前占位配置不会被保存。</span>
      <MacButton size="small" :disabled="initializing" @click="emit('retry')">重新读取</MacButton>
    </div>

    <!-- 时区设置中枢 -->
    <div class="settings-section settings-section--first">
      <div class="section-heading">
        <MacLabel id="settings-title" as="h2" variant="headline">时区核心配置</MacLabel>
        <p>选择目标运行地区或自定义时差偏移，右侧仪表盘将实时同步。</p>
      </div>

      <SettingRow label="模式选择">
        <MacSegmentedControl
          :model-value="settings.mode"
          type="single"
          size="regular"
          aria-label="设置方式"
          :disabled="busy || initializing || bootstrapFailed"
          @update:model-value="setMode"
        >
          <MacSegment value="zone">全球标准地区时区</MacSegment>
          <MacSegment value="offset">固定 UTC 偏移</MacSegment>
        </MacSegmentedControl>
      </SettingRow>

      <SettingRow
        v-if="settings.mode === 'zone'"
        label="目标时区"
        description="按国际 IANA 规则全自动处理夏令时与冬令时切换。"
        :error="zoneError"
      >
        <AppSelect
          :model-value="settings.zoneId"
          :items="zoneItems"
          :disabled="busy || initializing || bootstrapFailed"
          teleport-to="#timezone-menu-root"
          accessible-name="时区名称"
          :force-open="previewMenu === 'timezone'"
          @update:model-value="setZone"
        />
        <template #after>
          <p class="current-selection-tip">当前选定：<strong>{{ selectedZoneLabel }}</strong></p>
        </template>
      </SettingRow>

      <SettingRow
        v-else
        label="UTC 偏移量"
        description="选择与世界协调时 (UTC) 的固定绝对时差（-12 至 +14 小时）。"
      >
        <AppSelect
          :model-value="settings.offset"
          :items="offsetItems"
          :disabled="busy || initializing || bootstrapFailed"
          teleport-to="#offset-menu-root"
          accessible-name="固定 UTC 偏移"
          :force-open="previewMenu === 'offset'"
          @update:model-value="setOffset"
        />
        <div class="offset-summary">
          <span>当前固定偏移</span>
          <strong>{{ currentOffset }}</strong>
        </div>
      </SettingRow>
    </div>

    <!-- 客户端应用关联 -->
    <div class="settings-section">
      <div class="section-heading">
        <MacLabel as="h2" variant="headline">Codex 客户端程序</MacLabel>
        <p>支持 Microsoft Store (UWP) 自动探测或本地独立安装版路径。</p>
      </div>

      <SettingRow label="程序路径">
        <div class="path-row">
          <MacTextField
            :model-value="settings.executable"
            :placeholder="detected || '正在自动查找 Codex 桌面客户端…'"
            aria-label="Codex 客户端路径"
            :disabled="busy || initializing || bootstrapFailed"
            @update:model-value="update({ executable: $event })"
          />
          <MacButton :disabled="busy || initializing || bootstrapFailed || previewMode" title="开发预览不打开系统选择器" @click="emit('browse')">
            <AppIcon name="folder" />浏览…
          </MacButton>
        </div>
        <template #after>
          <div class="path-status-card" :class="{ 'path-status-card--found': detected || settings.executable, 'path-status-card--missing': !detected && !settings.executable }">
            <AppIcon :name="detected || settings.executable ? 'check' : 'alert'" />
            <div class="path-status-text">
              <strong>{{ settings.executable ? '已指定自定义安装路径' : detected ? '已自动识别系统安装版本' : '未检测到客户端' }}</strong>
              <small>{{ settings.executable || detected || '请点击右侧“浏览”手动指定 Codex.exe 或 ChatGPT.exe 安装路径。' }}</small>
            </div>
          </div>
        </template>
      </SettingRow>

      <!-- 系统托盘驻留提示卡片 -->
      <div class="tray-tip-card">
        <div class="tray-tip-icon">
          <AppIcon name="info" />
        </div>
        <div class="tray-tip-content">
          <strong>系统托盘常驻已就绪</strong>
          <p>点击窗口右上角关闭按钮将自动隐藏到 Windows 任务栏右下角托盘；随时右键托盘图标可一键快启或退出。</p>
        </div>
      </div>
    </div>
  </section>
</template>
