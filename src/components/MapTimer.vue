<template>
  <DraggableWidget widgetId="mapTimer" title="Map Timer" :defaultPosition="props.defaultPosition">
    <div
      class="flex flex-col justify-center items-center pointer-events-none w-full h-full text-center overflow-hidden"
      style="font-family: var(--theme-font-main); min-width: 90px;"
    >
      <div
        class="font-bold uppercase tracking-widest mb-1 select-none whitespace-nowrap overflow-hidden text-ellipsis w-full"
        style="color: var(--theme-text-qty); font-size: clamp(8px, min(calc(var(--widget-w, 120) * 0.10px), calc(var(--widget-h, 80) * 0.125px)), 28px); line-height: 1.1;"
      >MAP TIMER</div>

      <div
        class="font-mono font-bold tracking-[0.1em] select-none whitespace-nowrap overflow-hidden text-ellipsis w-full"
        style="color: var(--theme-text-header); font-size: clamp(14px, min(calc(var(--widget-w, 120) * 0.22px), calc(var(--widget-h, 80) * 0.35px)), 96px); line-height: 1.2;"
      >
        {{ store.formattedMapTimer }}
      </div>
    </div>
  </DraggableWidget>
</template>

<script setup lang="ts">
import { useTrackerStore } from '../stores/trackerStore';
import DraggableWidget from './DraggableWidget.vue';

const props = defineProps<{ defaultPosition?: { x: number; y: number } }>();
const store = useTrackerStore();
</script>
