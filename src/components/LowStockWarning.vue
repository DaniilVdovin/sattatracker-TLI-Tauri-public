<template>
  <Transition name="warn-slide">
    <div
      v-if="store.lowStockEnabled && store.lowStockWarnings.length > 0"
      class="fixed pointer-events-none select-none"
      style="bottom: 24px; left: 50%; transform: translateX(-50%); z-index: 200;"
    >
      <div
        class="flex flex-col gap-1 px-4 py-3 rounded-xl"
        style="
          background: linear-gradient(135deg, rgba(20,10,10,0.97) 0%, rgba(30,12,12,0.97) 100%);
          border: 1px solid rgba(239,68,68,0.5);
          box-shadow: 0 0 0 1px rgba(239,68,68,0.15), 0 4px 24px rgba(0,0,0,0.7), 0 0 20px rgba(239,68,68,0.12);
          min-width: 220px;
        "
      >
        <div class="flex items-center gap-2 pb-1" style="border-bottom: 1px solid rgba(239,68,68,0.2);">
          <span style="font-size: 13px;">⚠️</span>
          <span
            class="text-[11px] font-bold uppercase tracking-[2px]"
            style="color: rgba(239,68,68,0.9);"
          >Low Stock</span>
        </div>

        <div
          v-for="w in store.lowStockWarnings"
          :key="w.item_id"
          class="flex items-center justify-between gap-4"
        >
          <span
            class="text-[12px] font-medium truncate"
            style="color: rgba(255,255,255,0.75); max-width: 160px;"
          >{{ w.item_name || w.item_id }}</span>
          <span
            class="text-[13px] font-black tabular-nums"
            :style="{ color: w.qty === 1 ? '#ef4444' : '#f97316' }"
          >×{{ w.qty }}</span>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { useTrackerStore } from '../stores/trackerStore';
const store = useTrackerStore();
</script>

<style scoped>
.warn-slide-enter-active { transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1); }
.warn-slide-leave-active { transition: all 0.2s ease-in; }
.warn-slide-enter-from   { opacity: 0; transform: translateX(-50%) translateY(16px) scale(0.92); }
.warn-slide-leave-to     { opacity: 0; transform: translateX(-50%) translateY(16px) scale(0.92); }
</style>
