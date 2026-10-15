<template>
  <DraggableWidget widgetId="mapStats" title="Map Stats">
    <div
      class="flex flex-col space-y-2 pointer-events-none min-w-[200px] p-1 custom-scroll"
      style="font-family: var(--theme-font-main);"
    >
      <div
        class="flex justify-between items-center text-sm border-b pb-1"
        style="border-color: var(--theme-border-color);"
      >
         <span style="color: var(--theme-text-qty);">Current Zone:</span>
         <span class="font-bold truncate max-w-[120px]" style="color: var(--theme-text-main);">
            {{ store.currentZone }}
         </span>
      </div>
      <div class="flex justify-between items-center text-sm">
         <span style="color: var(--theme-text-qty);">Map Profit:</span>
         <span
            class="font-bold"
            :style="{ color: store.currentMapProfit >= 0 ? 'var(--theme-text-price)' : '#ff5555' }"
         >
            {{ store.currentMapProfit.toFixed(1) }} FE
         </span>
      </div>
      <div class="flex justify-between items-center text-sm">
         <span style="color: var(--theme-text-qty);">Map Time:</span>
         <span class="font-mono font-semibold" style="color: var(--theme-text-header);">
            {{ store.formattedMapTimer }}
         </span>
      </div>
    </div>
  </DraggableWidget>
</template>

<script setup lang="ts">
import { useTrackerStore } from '../stores/trackerStore';
import DraggableWidget from './DraggableWidget.vue';

const store = useTrackerStore();
</script>
