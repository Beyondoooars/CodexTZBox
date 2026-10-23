<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import AppIcon from './AppIcon.vue'
import { searchZones, zoneLabel, type ZoneOption } from '../../zones'

const props = withDefaults(defineProps<{
  modelValue: string
  disabled?: boolean
  accessibleName?: string
  defaultOpen?: boolean
}>(), {
  disabled: false,
  accessibleName: '目标时区',
  defaultOpen: false,
})

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

const root = ref<HTMLElement | null>(null)
const inputEl = ref<HTMLInputElement | null>(null)
const panelEl = ref<HTMLElement | null>(null)
const listEl = ref<HTMLElement | null>(null)
const open = ref(false)
const query = ref('')
const activeIndex = ref(0)
const panelStyle = ref<{ left: string; top: string; width: string; maxHeight: string }>({
  left: '0px',
  top: '0px',
  width: '0px',
  maxHeight: 'none',
})

// 不做截断：结果面板自身可滚动，截断会让排在后面的时区（如北美）在默认列表里
// 完全不可见，用户只能靠拼写搜索才找得到。
const results = computed<ZoneOption[]>(() => searchZones(query.value))
const displayValue = computed(() => {
  const label = zoneLabel(props.modelValue)
  return label ? `${label}（${props.modelValue}）` : props.modelValue
})

function reposition() {
  const el = inputEl.value
  if (!el) return
  const rect = el.getBoundingClientRect()
  const width = Math.max(rect.width, 320)
  const left = Math.min(rect.left, Math.max(12, window.innerWidth - width - 12))
  // 面板高度上限与 CSS 中的 max-height 保持一致
  const maxPanelHeight = Math.min(360, window.innerHeight * 0.5)
  const gap = 6
  const below = rect.bottom + gap
  const spaceBelow = window.innerHeight - below - 12
  const spaceAbove = rect.top - gap - 12
  // 下方放不下、且上方空间更充裕时，向上展开，避免遮挡并减少滚动
  const openUp = spaceBelow < Math.min(maxPanelHeight, 200) && spaceAbove > spaceBelow
  const top = openUp ? Math.max(12, rect.top - gap - maxPanelHeight) : below
  panelStyle.value = {
    left: `${Math.round(left)}px`,
    top: `${Math.round(top)}px`,
    width: `${Math.round(width)}px`,
    maxHeight: `${Math.round(openUp ? Math.min(maxPanelHeight, spaceAbove) : Math.min(maxPanelHeight, spaceBelow))}px`,
  }
}

function openMenu() {
  if (props.disabled || open.value) return
  open.value = true
  query.value = ''
  activeIndex.value = 0
  reposition()
  void nextTick(reposition)
}

function closeMenu() {
  if (!open.value) return
  open.value = false
  query.value = ''
}

function select(option: ZoneOption) {
  emit('update:modelValue', option.id)
  closeMenu()
}

function moveActive(delta: number) {
  if (!open.value) {
    openMenu()
    return
  }
  const total = results.value.length
  if (!total) return
  activeIndex.value = (activeIndex.value + delta + total) % total
  void nextTick(() => {
    const item = listEl.value?.children[activeIndex.value] as HTMLElement | undefined
    item?.scrollIntoView({ block: 'nearest' })
  })
}

function handleInput(event: Event) {
  query.value = (event.target as HTMLInputElement).value
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    moveActive(1)
    return
  }
  if (event.key === 'ArrowUp') {
    event.preventDefault()
    moveActive(-1)
    return
  }
  if (event.key === 'Enter') {
    event.preventDefault()
    if (!open.value) {
      openMenu()
      return
    }
    const option = results.value[activeIndex.value]
    if (option) {
      select(option)
      return
    }
    const exact = results.value.find((item) => item.id.toLowerCase() === query.value.trim().toLowerCase())
    if (exact) select(exact)
    return
  }
  if (event.key === 'Escape') {
    event.preventDefault()
    closeMenu()
    inputEl.value?.blur()
  }
}

function handleDocumentPointer(event: PointerEvent) {
  if (!open.value) return
  const target = event.target as Node | null
  if (target && (root.value?.contains(target) || panelEl.value?.contains(target))) return
  closeMenu()
}

watch(results, () => {
  activeIndex.value = 0
})

onMounted(() => {
  document.addEventListener('pointerdown', handleDocumentPointer, true)
  window.addEventListener('scroll', reposition, true)
  window.addEventListener('resize', reposition)
  if (props.defaultOpen && !props.disabled) openMenu()
})

onBeforeUnmount(() => {
  document.removeEventListener('pointerdown', handleDocumentPointer, true)
  window.removeEventListener('scroll', reposition, true)
  window.removeEventListener('resize', reposition)
})
</script>

<template>
  <div ref="root" class="zone-select">
    <div class="zone-select-field">
      <input
        ref="inputEl"
        class="zone-select-input"
        type="text"
        :value="open ? query : displayValue"
        :placeholder="open ? '搜索城市或时区，如 shanghai、东京' : ''"
        :disabled="disabled"
        :aria-label="accessibleName"
        role="combobox"
        :aria-expanded="open"
        :aria-controls="open ? 'zone-select-listbox' : undefined"
        :aria-activedescendant="open && results.length ? `zone-select-option-${activeIndex}` : undefined"
        autocomplete="off"
        spellcheck="false"
        @focus="openMenu"
        @input="handleInput"
        @keydown="handleKeydown"
        @blur="closeMenu()"
      />
      <AppIcon name="clock" :size="14" class="zone-select-icon" />
    </div>

    <Teleport to="#timezone-menu-root">
      <div
        v-if="open"
        ref="panelEl"
        class="zone-select-panel"
        :style="panelStyle"
        @mousedown.prevent
      >
        <ul v-if="results.length" id="zone-select-listbox" ref="listEl" class="zone-select-list" role="listbox">
          <li
            v-for="(option, index) in results"
            :id="`zone-select-option-${index}`"
            :key="option.id"
            class="zone-select-option"
            :class="{
              'zone-select-option--active': index === activeIndex,
              'zone-select-option--current': option.id === modelValue,
            }"
            role="option"
            :aria-selected="option.id === modelValue"
            @click="select(option)"
            @mousemove="activeIndex = index"
          >
            <span class="zone-select-option-label">{{ option.label }}</span>
            <span class="zone-select-option-detail">{{ option.id }}</span>
          </li>
        </ul>
        <div v-else class="zone-select-empty">未找到匹配的时区；可直接输入完整 IANA 名称，如 Asia/Kathmandu</div>
      </div>
    </Teleport>
  </div>
</template>
