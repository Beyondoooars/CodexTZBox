<script setup lang="ts">
import { ref } from 'vue'
import { MacButton, MacLabel, MacSpinner } from '@macvue/core'
import type { IpInfo, NetworkInfo } from '../types'
import AppIcon from './ui/AppIcon.vue'
import NetworkRegion from './NetworkRegion.vue'

defineProps<{
  network: NetworkInfo | null
  networkBusy: boolean
  hasCachedNetwork: boolean
  disabled: boolean
  copyFeedback: Record<'domestic' | 'international', 'idle' | 'copied' | 'error'>
  copyErrors: Partial<Record<'domestic' | 'international', string>>
}>()
const emit = defineEmits<{
  refresh: []
  expand: []
  copy: [key: 'domestic' | 'international', ip: string]
  use: [info: IpInfo]
}>()

const isExpanded = ref(false)

function toggleExpanded() {
  const next = !isExpanded.value
  isExpanded.value = next
  if (next) emit('expand')
}

function forwardCopy(key: 'domestic' | 'international', ip: string) {
  emit('copy', key, ip)
}
</script>

<template>
  <section class="surface network-panel" :class="{ 'network-panel--collapsed': !isExpanded }" aria-labelledby="network-title">
    <div class="panel-heading panel-heading--collapsible" @click="toggleExpanded">
      <div class="panel-heading-title">
        <div class="heading-with-icon">
          <AppIcon name="refresh" :size="13" />
          <MacLabel id="network-title" as="h2" variant="headline">网络出口</MacLabel>
        </div>
        <p>探测公网出口 IP 与归属地时区，可一键填入</p>
      </div>
      <div class="panel-heading-actions" @click.stop>
        <MacButton size="small" :disabled="networkBusy" @click="emit('refresh')">
          <MacSpinner v-if="networkBusy" size="small" />
          <AppIcon v-else name="refresh" />
          {{ networkBusy ? '探测中…' : '探测网络' }}
        </MacButton>
        <MacButton size="small" @click="toggleExpanded">
          {{ isExpanded ? '收起' : '详情' }}
        </MacButton>
      </div>
    </div>
    <div v-show="isExpanded" class="network-grid">
      <NetworkRegion
        region-key="domestic"
        title="国内出口"
        description="访问国内服务检测到的公网出口"
        :info="network?.domestic || null"
        :loading="networkBusy"
        :cached="hasCachedNetwork"
        :disabled="disabled"
        :copy-state="copyFeedback.domestic"
        :copy-error="copyErrors.domestic"
        @copy="forwardCopy"
        @use="emit('use', $event)"
      />
      <NetworkRegion
        region-key="international"
        title="国际出口"
        description="访问国际服务检测到的公网出口"
        :info="network?.international || null"
        :loading="networkBusy"
        :cached="hasCachedNetwork"
        :disabled="disabled"
        :copy-state="copyFeedback.international"
        :copy-error="copyErrors.international"
        @copy="forwardCopy"
        @use="emit('use', $event)"
      />
    </div>
  </section>
</template>
