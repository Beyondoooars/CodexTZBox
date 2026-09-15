<script setup lang="ts">
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
  copy: [key: 'domestic' | 'international', ip: string]
  use: [info: IpInfo]
}>()

function forwardCopy(key: 'domestic' | 'international', ip: string) {
  emit('copy', key, ip)
}
</script>

<template>
  <section class="surface network-panel" aria-labelledby="network-title">
    <div class="panel-heading">
      <div>
        <MacLabel id="network-title" as="h2" variant="headline">网络出口与时区感知</MacLabel>
        <p>独立检测国内与国际出口公网 IP 及地理归属地，可一键将识别时区填入设置草稿。</p>
      </div>
      <MacButton :disabled="networkBusy" @click="emit('refresh')">
        <MacSpinner v-if="networkBusy" size="small" />
        <AppIcon v-else name="refresh" />
        {{ networkBusy ? '正在探测…' : '重新探测网络' }}
      </MacButton>
    </div>
    <div class="network-grid">
      <NetworkRegion
        region-key="domestic"
        title="国内出口"
        description="访问国内服务时检测到的公网出口"
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
        description="访问国际服务时检测到的公网出口"
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
