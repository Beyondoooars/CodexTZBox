<script setup lang="ts">
import { computed } from 'vue'
import { MacLabel } from '@macvue/core'
import type { ClockValue, Settings, Zone } from '../types'
import AppIcon from './ui/AppIcon.vue'

const props = defineProps<{
  settings: Settings
  zones: Zone[]
  targetClock: ClockValue
  localClock: ClockValue
  localZone: string
  platform: string
}>()

const targetName = computed(() => {
  if (props.settings.mode === 'offset') return `固定偏移 ${props.targetClock.offset}`
  const zone = props.zones.find((item) => item.id === props.settings.zoneId)
  if (!zone || zone.label === zone.id) return props.settings.zoneId
  return `${zone.label} · ${zone.id}`
})

const diffInfo = computed(() => {
  const diff = props.targetClock.offsetMinutes - props.localClock.offsetMinutes
  if (diff === 0) return { text: '与本地时间一致', type: 'same' }
  const totalMins = Math.abs(diff)
  const hours = Math.floor(totalMins / 60)
  const mins = totalMins % 60
  const timeStr = mins > 0 ? `${hours}小时${mins}分` : `${hours}小时`
  return {
    text: diff > 0 ? `比本地快 ${timeStr}` : `比本地慢 ${timeStr}`,
    type: diff > 0 ? 'faster' : 'slower',
  }
})
</script>

<template>
  <section class="surface preview-panel" aria-labelledby="preview-title">
    <div class="section-heading preview-header">
      <div>
        <MacLabel id="preview-title" as="h2" variant="headline">时空对照仪表盘</MacLabel>
        <p>目标 Codex 进程时区与本地环境实时同步对照</p>
      </div>
      <div class="diff-badge" :class="`diff-badge--${diffInfo.type}`">
        <AppIcon name="clock" />
        <span>{{ diffInfo.text }}</span>
      </div>
    </div>

    <!-- 目标时间（主视觉卡片） -->
    <div class="clock-card clock-card--target">
      <div class="clock-card-header">
        <span class="clock-tag clock-tag--target">
          <span class="pulse-dot"></span>
          目标生效时间
        </span>
        <span class="clock-offset-pill">{{ targetClock.offset }}</span>
      </div>
      <div class="clock-time-display">
        <time :datetime="targetClock.time" class="hero-digits">{{ targetClock.time }}</time>
      </div>
      <div class="clock-card-meta">
        <p class="clock-date-text">{{ targetClock.date }}</p>
        <div class="clock-zone-row">
          <strong class="target-zone-name">{{ targetName }}</strong>
          <span class="clock-rule-tag">{{ settings.mode === 'zone' ? 'IANA 夏令时规则' : '固定 UTC 偏移' }}</span>
        </div>
      </div>
    </div>

    <!-- 本地时间（参考卡片） -->
    <div class="clock-card clock-card--local">
      <div class="clock-card-header">
        <span class="clock-tag clock-tag--local">Windows 系统本地时区</span>
        <span class="clock-offset-pill clock-offset-pill--muted">{{ localClock.offset }}</span>
      </div>
      <div class="clock-time-display">
        <time :datetime="localClock.time" class="secondary-digits">{{ localClock.time }}</time>
      </div>
      <div class="clock-card-meta">
        <p class="clock-date-text">{{ localClock.date }}</p>
        <div class="clock-zone-row">
          <strong class="local-zone-name">{{ localZone }}</strong>
          <span class="clock-rule-tag clock-rule-tag--muted">宿主操作系统</span>
        </div>
      </div>
    </div>
  </section>
</template>
