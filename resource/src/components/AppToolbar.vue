<script setup lang="ts">
import { MacLabel } from '@macvue/core'
import AppIcon from './ui/AppIcon.vue'
</script>

<template>
  <header class="app-toolbar">
    <div class="toolbar-inner">
      <div class="brand-group">
        <div class="brand-mark"><AppIcon name="clock" :size="18" /></div>
        <div class="toolbar-title">
          <MacLabel as="h1" variant="headline">Codex 时区沙盒</MacLabel>
          <p>进程级独立时区隔离 · 宿主系统时钟零干扰</p>
        </div>
      </div>
    </div>
  </header>
</template>
