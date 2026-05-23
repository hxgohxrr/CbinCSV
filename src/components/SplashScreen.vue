<template>
  <div class="splash" :class="{ leaving }" @click="skip">
    <img class="splash-logo" src="../assets/logo.png" alt="CBinCSV" />
    <div class="splash-bar">
      <div class="splash-fill" :style="{ width: progress + '%' }" />
    </div>
    <span class="splash-hint">click para continuar</span>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const emit = defineEmits<{ done: [] }>()

const progress = ref(0)
const leaving = ref(false)

let startTime = 0
let rafId = 0
let leaveTimer = 0
const DURATION = 2000
const FADE = 350

function tick() {
  const elapsed = Date.now() - startTime
  progress.value = Math.min((elapsed / DURATION) * 100, 100)
  if (elapsed < DURATION) {
    rafId = requestAnimationFrame(tick)
  } else {
    startLeave()
  }
}

function startLeave() {
  cancelAnimationFrame(rafId)
  leaving.value = true
  leaveTimer = window.setTimeout(() => emit('done'), FADE)
}

function skip() {
  startLeave()
}

onMounted(() => {
  startTime = Date.now()
  rafId = requestAnimationFrame(tick)
})

onUnmounted(() => {
  cancelAnimationFrame(rafId)
  clearTimeout(leaveTimer)
})
</script>

<style scoped>
.splash {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: var(--bg-base);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 28px;
  cursor: pointer;
  opacity: 1;
  transition: opacity 0.35s ease;
}
.splash.leaving { opacity: 0; pointer-events: none; }

.splash-logo {
  width: 220px;
  height: auto;
  object-fit: contain;
}

.splash-bar {
  width: 180px;
  height: 2px;
  background: var(--border-md);
  border-radius: 2px;
  overflow: hidden;
}
.splash-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 2px;
  transition: width 0.05s linear;
}

.splash-hint {
  font-size: 10px;
  color: var(--text-3);
  letter-spacing: 0.06em;
  text-transform: uppercase;
  position: absolute;
  bottom: 20px;
}
</style>
