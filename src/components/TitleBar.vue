<template>
  <header class="titlebar" :class="{ 'is-mac': isMac }" data-tauri-drag-region>
    <div class="logo" data-tauri-drag-region>
      <img :src="appIcon" class="logo-mark" alt="" />
      <span class="logo-name">CBin<em>CSV</em></span>
    </div>

    <div class="spacer" data-tauri-drag-region />

    <div v-if="!isMac" class="win-controls">
      <button class="wc-btn" @click="minimize">
        <svg viewBox="0 0 10 1" fill="currentColor"><rect width="10" height="1"/></svg>
      </button>
      <button class="wc-btn" @click="toggleMaximize">
        <svg viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1">
          <rect x="0.5" y="0.5" width="9" height="9"/>
        </svg>
      </button>
      <button class="wc-btn wc-close" @click="close">
        <svg viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.2">
          <line x1="1" y1="1" x2="9" y2="9"/><line x1="9" y1="1" x2="1" y2="9"/>
        </svg>
      </button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'
import appIcon from '../assets/logo.png'

const isMac = /Macintosh|MacIntel/.test(navigator.userAgent)

const win = getCurrentWindow()
async function minimize() { await win.minimize() }
async function toggleMaximize() { await win.toggleMaximize() }
async function close() { await win.close() }
</script>

<style scoped>
.titlebar {
  height: 40px;
  background: var(--bg-panel);
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  padding: 0 0 0 16px;
  gap: 8px;
  flex-shrink: 0;
  user-select: none;
}
.titlebar.is-mac { padding-left: 80px; }

.logo { display: flex; align-items: center; gap: 8px; flex-shrink: 0; }
.logo-mark {
  width: 20px; height: 20px;
  flex-shrink: 0;
  object-fit: contain;
}
.logo-name { font-size: 12px; font-weight: 600; letter-spacing: 0.04em; }
.logo-name em { font-style: normal; color: var(--accent); }

.spacer { flex: 1; }

.win-controls {
  display: flex; align-items: stretch; height: 100%;
  flex-shrink: 0; border-left: 1px solid var(--border);
}
.wc-btn {
  width: 46px; height: 100%;
  background: transparent; border: none; color: var(--text-2);
  cursor: pointer; display: flex; align-items: center; justify-content: center;
  transition: background 0.1s, color 0.1s;
}
.wc-btn svg { width: 10px; height: 10px; }
.wc-btn:hover { background: var(--bg-hover); color: var(--text-1); }
.wc-close:hover { background: #C42B1C; color: #fff; }
</style>
