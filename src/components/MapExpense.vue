<template>
  <DraggableWidget widgetId="mapExpense" title="Map Expenses" :defaultPosition="props.defaultPosition" :defaultW="220" :defaultH="160">
    <div
      class="flex flex-col pointer-events-none w-full h-full"
      style="font-family: var(--theme-font-main); min-width: 140px; min-height: 60px;"
    >
      <div class="flex-1 overflow-y-auto custom-scroll px-[6px] pt-[4px] flex flex-col gap-[3px]">
        <div
          v-if="store.mapExpenses.length === 0"
          class="flex items-center justify-center h-full"
          style="color: var(--theme-text-qty); font-size: clamp(9px, calc(var(--widget-w, 220) * 0.048px), 13px);"
        >
          No expenses yet
        </div>

        <div
          v-for="item in store.mapExpenses"
          :key="item.id"
          class="flex items-center justify-between rounded px-[6px] py-[2px] shrink-0"
          style="background: rgba(0,0,0,0.35); border: 1px solid rgba(248,113,113,0.2);"
        >
          <span
            class="font-bold truncate text-red-200"
            style="font-size: clamp(9px, calc(var(--widget-w, 220) * 0.052px), 13px);"
          >{{ item.name }}</span>
          <span
            class="font-bold text-red-500 whitespace-nowrap ml-2 shrink-0"
            style="font-size: clamp(9px, calc(var(--widget-w, 220) * 0.052px), 13px);"
          >x{{ item.qty }} &minus;{{ item.total.toFixed(1) }}</span>
        </div>
      </div>

      <div
        class="shrink-0 border-t px-[6px] py-[3px] font-bold text-right text-red-500"
        style="border-color: var(--theme-border-color); font-size: clamp(9px, calc(var(--widget-w, 220) * 0.058px), 14px);"
      >
        Cost: {{ totalCost.toFixed(1) }} FE
      </div>
    </div>
  </DraggableWidget>
</template>

<script setup lang="ts">
import { computed } from 'vue';
const props = defineProps<{ defaultPosition?: { x: number, y: number } }>();
import { useTrackerStore } from '../stores/trackerStore';
import DraggableWidget from './DraggableWidget.vue';

const store = useTrackerStore();

const totalCost = computed(() =>
  store.mapExpenses.reduce((sum, e) => sum + e.total, 0)
);
</script>
