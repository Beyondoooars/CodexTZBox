<script setup lang="ts">
import { computed } from 'vue'
import { MacButton, MacLabel, MacSegment, MacSegmentedControl, MacTextField } from '@macvue/core'
import type { Settings } from '../types'
import { offsetText } from '../composables/useClocks'
import AppIcon from './ui/AppIcon.vue'
import AppSelect, { type AppSelectItem } from './ui/AppSelect.vue'
import SettingRow from './ui/SettingRow.vue'
import ZoneSelect from './ui/ZoneSelect.vue'

const props = defineProps<{
  settings: Settings
  detected: string
  busy: boolean
  initializing: boolean
  bootstrapFailed: boolean
  zoneError: string
  previewMode: boolean
}>()

const emit = defineEmits<{
  'update:settings': [settings: Settings]
  browse: []
  retry: []
}>()

const offsets = Array.from({ length: 27 }, (_, index) => index - 12)
const offsetItems = computed<AppSelectItem[]>(() => offsets.map((offset) => ({ value: offset, label: offsetText(offset * 60) })))
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
  <div class="settings-stack">
    <!-- 失败重试通知 -->
    <div v-if="bootstrapFailed" class="inline-alert" role="alert">
      <AppIcon name="alert" />
      <span>配置读取失败，当前配置无法保存。</span>
      <MacButton size="small" :disabled="initializing" @click="emit('retry')">重试</MacButton>
    </div>

    <!-- 卡片 1：时区核心配置与实时对比 -->
    <section class="surface settings-panel" aria-labelledby="timezone-card-title">
      <div class="section-heading">
        <MacLabel id="timezone-card-title" as="h2" variant="headline">时区</MacLabel>
      </div>

      <SettingRow label="配置模式">
        <MacSegmentedControl
          :model-value="settings.mode"
          type="single"
          size="regular"
          aria-label="设置方式"
          :disabled="busy || initializing || bootstrapFailed"
          @update:model-value="setMode"
        >
          <MacSegment value="zone">地区时区</MacSegment>
          <MacSegment value="offset">固定偏移</MacSegment>
        </MacSegmentedControl>
      </SettingRow>

      <SettingRow
        v-if="settings.mode === 'zone'"
        label="目标时区"
        :error="zoneError"
      >
        <ZoneSelect
          :model-value="settings.zoneId"
          :disabled="busy || initializing || bootstrapFailed"
          :default-open="previewMenu === 'timezone'"
          @update:model-value="setZone"
        />
      </SettingRow>

      <SettingRow
        v-else
        label="UTC 偏移量"
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
      </SettingRow>

      <!-- 紧凑实时双时钟对比条 (Slot) -->
      <slot name="preview" />
    </section>

    <!-- 卡片 2：Codex 客户端程序 -->
    <section class="surface settings-panel" aria-labelledby="client-card-title">
      <div class="section-heading">
        <MacLabel id="client-card-title" as="h2" variant="headline">客户端</MacLabel>
      </div>

      <div class="path-row">
        <MacTextField
          :model-value="settings.executable"
          :placeholder="detected || '正在自动查找 Codex 桌面客户端…'"
          aria-label="Codex 客户端路径"
          :disabled="busy || initializing || bootstrapFailed"
          @update:model-value="update({ executable: $event })"
        />
        <MacButton :disabled="busy || initializing || bootstrapFailed || previewMode" title="浏览文件" @click="emit('browse')">
          <AppIcon name="folder" />浏览…
        </MacButton>
      </div>

      <div class="path-status-line">
        <span
          class="status-dot"
          :class="detected || settings.executable ? 'status-dot--ok' : 'status-dot--warn'"
        ></span>
        <span class="status-text">
          {{
            settings.executable
              ? '已使用自定义路径：' + settings.executable
              : detected
                ? '已自动识别系统客户端：' + detected
                : '未检测到客户端，请点击“浏览”手动选择 Codex.exe 安装路径'
          }}
        </span>
      </div>
    </section>
  </div>
</template>
