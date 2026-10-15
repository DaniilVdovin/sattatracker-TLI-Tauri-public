<template>
  <DraggableWidget widgetId="controlPanel" title="Control Panel" :defaultPosition="{ x: 20, y: 20 }" :alwaysInteractive="true">
    <div class="flex items-center gap-[6px]">
      <button
        class="pointer-events-auto flex items-center gap-[5px] px-4 py-[6px] rounded-xl text-[12px] font-bold transition-all duration-150"
        :style="{
          background: store.isTimerRunning ? 'rgba(248,113,113,0.13)' : 'rgba(74,222,128,0.13)',
          border:     store.isTimerRunning ? '1px solid rgba(248,113,113,0.35)' : '1px solid rgba(74,222,128,0.35)',
          color:      store.isTimerRunning ? '#f87171' : '#4ade80',
        }"
        @click.stop="store.isTimerRunning ? store.pauseSession() : store.resumeSession()"
      >
        <span>{{ store.isTimerRunning ? '⏸' : '▶' }}</span>
        <span>{{ store.isTimerRunning ? 'Stop' : 'Start' }}</span>
      </button>

      <button
        class="pointer-events-auto flex items-center gap-[5px] px-4 py-[6px] rounded-xl text-[12px] font-bold transition-all duration-150"
        style="background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); color: rgba(255,255,255,0.45);"
        onmouseover="this.style.background='rgba(255,255,255,0.1)'; this.style.color='rgba(255,255,255,0.75)'"
        onmouseout="this.style.background='rgba(255,255,255,0.05)'; this.style.color='rgba(255,255,255,0.45)'"
        @click.stop="store.resetSession()"
      >
        <span>↺</span>
        <span>Reset</span>
      </button>
    </div>
  </DraggableWidget>
</template>

<script setup lang="ts">
import { useTrackerStore } from '../stores/trackerStore';
import DraggableWidget from './DraggableWidget.vue';

const store = useTrackerStore();
</script>
