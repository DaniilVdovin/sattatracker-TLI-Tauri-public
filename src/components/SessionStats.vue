<template>
  <DraggableWidget widgetId="sessionStats" title="Session Overview">
    <div
      class="flex flex-col space-y-2 pointer-events-none min-w-[200px] p-1 custom-scroll"
      style="font-family: var(--theme-font-main);"
    >
      <div
        class="flex justify-between items-center text-sm border-b pb-1"
        style="border-color: var(--theme-border-color);"
      >
         <span style="color: var(--theme-text-qty);">Total Profit:</span>
         <span
            class="font-bold text-lg"
            :style="{ color: store.totalProfit >= 0 ? 'var(--theme-text-price)' : '#ff5555' }"
         >
            {{ store.totalProfit.toFixed(1) }} FE
         </span>
      </div>
      <div class="flex justify-between items-center text-sm">
         <span style="color: var(--theme-text-qty);">FE / Hour:</span>
         <span class="font-bold" style="color: var(--theme-text-header);">
            {{ store.profitPerHour }} FE/h
         </span>
      </div>
      <div class="flex justify-between items-center text-sm">
         <span style="color: var(--theme-text-qty);">Session Time:</span>
         <span class="font-mono font-semibold" style="color: var(--theme-text-main);">
            {{ store.formattedSessionTimer }}
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
