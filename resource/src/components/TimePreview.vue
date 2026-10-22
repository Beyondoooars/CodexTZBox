<script setup lang="ts">
import { computed } from 'vue'
import type { ClockValue, Settings, Zone } from '../types'
import { zoneLabel } from '../zones'
import AppIcon from './ui/AppIcon.vue'

const props = defineProps<{
  settings: Settings
  zones: Zone[]
  targetClock: ClockValue
  localClock: ClockValue
  localZone: string
}>()

const targetName = computed(() => {
  // targetClock.offset 已自带 “UTC ±HH:MM”，不要再拼一次前缀。
  if (props.settings.mode === 'offset') return `固定偏移 ${props.targetClock.offset}`
  const zone = props.zones.find((item) => item.id === props.settings.zoneId)
  if (zone && zone.label !== zone.id) return `${zone.label} · ${zone.id}`
  const label = zoneLabel(props.settings.zoneId)
  return label ? `${label} · ${props.settings.zoneId}` : props.settings.zoneId
})

const diffInfo = computed(() => {
  const diff = props.targetClock.offsetMinutes - props.localClock.offsetMinutes
  if (diff === 0) return { text: '与本地一致', type: 'same' }
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
  <div class="compact-clock-bar" aria-label="实时双时钟对比">
    <div class="clock-cell clock-cell--target">
      <div class="clock-cell-header">
        <span class="clock-cell-badge clock-cell-badge--target">
          <span class="pulse-dot"></span>Codex 进程
        </span>
        <span class="clock-offset-tag">{{ targetClock.offset }}</span>
      </div>
      <div class="clock-time-val">{{ targetClock.time }}</div>
      <div class="clock-meta-val" :title="targetName">{{ targetName }}</div>
      <div class="clock-meta-val" :title="targetClock.date">{{ targetClock.date }}</div>
    </div>

    <div class="clock-divider">
      <div class="diff-badge" :class="`diff-badge--${diffInfo.type}`" :title="diffInfo.text">
        <AppIcon name="clock" :size="12" />
        <span>{{ diffInfo.text }}</span>
      </div>
    </div>

    <div class="clock-cell clock-cell--local">
      <div class="clock-cell-header">
        <span class="clock-offset-tag clock-offset-tag--muted">{{ localClock.offset }}</span>
        <span class="clock-cell-badge clock-cell-badge--local">本机系统</span>
      </div>
      <div class="clock-time-val">{{ localClock.time }}</div>
      <div class="clock-meta-val" :title="localZone">{{ localZone }}</div>
      <div class="clock-meta-val" :title="localClock.date">{{ localClock.date }}</div>
    </div>
  </div>
</template>
