import { computed, onBeforeUnmount, onMounted, ref, type Ref } from 'vue'
import type { ClockValue, Settings } from '../types'

const dateTimeFormatters = new Map<string, Intl.DateTimeFormat>()

function formatter(locale: 'zh-CN' | 'en-US', zone: string) {
  const key = `${locale}:${zone}`
  let value = dateTimeFormatters.get(key)
  if (!value) {
    value = new Intl.DateTimeFormat(locale, {
      timeZone: zone,
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      weekday: locale === 'zh-CN' ? 'long' : undefined,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      hourCycle: 'h23',
    })
    dateTimeFormatters.set(key, value)
  }
  return value
}

function parts(date: Date, zone: string) {
  return Object.fromEntries(formatter('zh-CN', zone).formatToParts(date).map((item) => [item.type, item.value]))
}

function offsetFor(date: Date, zone: string) {
  const value = Object.fromEntries(formatter('en-US', zone).formatToParts(date).map((item) => [item.type, item.value]))
  return Math.round((Date.UTC(+value.year, +value.month - 1, +value.day, +value.hour, +value.minute, +value.second) - date.getTime()) / 60_000)
}

export function offsetText(minutes: number) {
  const sign = minutes < 0 ? '−' : '+'
  const value = Math.abs(minutes)
  return `UTC ${sign}${String(Math.floor(value / 60)).padStart(2, '0')}:${String(value % 60).padStart(2, '0')}`
}

// 判断是否为浏览器 Intl 能识别的时区名（含 EST、CST 这类缩写别名）。
function isIntlTimeZone(zone: string) {
  try {
    new Intl.DateTimeFormat('zh-CN', { timeZone: zone }).format()
    return true
  } catch {
    return false
  }
}

// 与后端 common.rs 的 is_valid_iana_zone 保持一致：UTC/GMT 或至少含一级 “区域/地点”，
// 且每段只允许字母数字与 _ - +。这样前端不会放过后端要拒绝的缩写别名（EST、CST）。
export function isValidIanaZone(zone: string) {
  const value = zone.trim()
  if (!value || value.length > 64) return false
  if (value === 'UTC' || value === 'GMT') return true
  const parts = value.split('/')
  if (parts.length < 2) return false
  return parts.every((part) => part.length > 0 && /^[A-Za-z0-9_+-]+$/.test(part))
}

export function isValidTimeZone(zone: string) {
  return isValidIanaZone(zone) && isIntlTimeZone(zone)
}

export function useClocks(settings: Ref<Settings>, localZone: Ref<string>) {
  const previewMode = import.meta.env.DEV && new URLSearchParams(window.location.search).get('ui-preview') === '1'
  const now = ref(previewMode ? new Date('2026-09-08T09:24:08Z') : new Date())
  let clockTimer = 0

  function clock(zone: string, fixed?: number): ClockValue {
    const safeZone = isValidTimeZone(zone) ? zone : 'UTC'
    const date = fixed === undefined ? now.value : new Date(now.value.getTime() + fixed * 3_600_000)
    const value = parts(date, fixed === undefined ? safeZone : 'UTC')
    const minutes = fixed === undefined ? offsetFor(now.value, safeZone) : fixed * 60
    return {
      time: `${value.hour}:${value.minute}:${value.second}`,
      date: `${value.year} 年 ${value.month} 月 ${value.day} 日　${value.weekday}`,
      offset: offsetText(minutes),
      offsetMinutes: minutes,
    }
  }

  const targetClock = computed(() => settings.value.mode === 'offset'
    ? clock('UTC', settings.value.offset)
    : clock(settings.value.zoneId))
  const localClock = computed(() => clock(localZone.value))

  function tickClock() {
    window.clearTimeout(clockTimer)
    now.value = new Date()
    if (!document.hidden) clockTimer = window.setTimeout(tickClock, 1000 - (Date.now() % 1000) + 8)
  }

  function handleVisibilityChange() {
    if (document.hidden) window.clearTimeout(clockTimer)
    else tickClock()
  }

  onMounted(() => {
    if (previewMode) return
    document.addEventListener('visibilitychange', handleVisibilityChange)
    tickClock()
  })

  onBeforeUnmount(() => {
    if (previewMode) return
    window.clearTimeout(clockTimer)
    document.removeEventListener('visibilitychange', handleVisibilityChange)
  })

  return { targetClock, localClock }
}
