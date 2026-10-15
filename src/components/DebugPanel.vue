<template>
  <div class="fixed top-4 left-1/2 transform -translate-x-1/2 bg-gray-900 bg-opacity-95 text-white p-4 rounded-xl shadow-2xl border-2 border-purple-500 z-[100] flex flex-col gap-3 pointer-events-auto min-w-[300px]" v-if="isVisible">

    <div class="flex justify-between items-center border-b border-gray-700 pb-2">
      <h2 class="text-xl font-bold text-purple-400">🕵️ Debug Panel</h2>
      <button @click="isVisible = false" class="text-gray-400 hover:text-white bg-transparent border-none cursor-pointer">✕</button>
    </div>

    <div class="flex flex-col gap-2 bg-gray-800 p-2 rounded-lg">
      <h3 class="text-sm font-semibold uppercase text-gray-400">Loot Simulator</h3>
      <button @click="spawnLoot(2, 'trash_item')" class="bg-gray-700 hover:bg-gray-600 px-3 py-2 rounded text-sm transition-colors text-left border-l-4 border-gray-400">🗑️ Low Tier (2 FE)</button>
      <button @click="spawnLoot(25, 'mid_item')" class="bg-gray-700 hover:bg-gray-600 px-3 py-2 rounded text-sm transition-colors text-left border-l-4 border-green-500">💰 Mid Tier (25 FE)</button>
      <button @click="spawnLoot(150, 'jackpot_item')" class="bg-gray-700 hover:bg-gray-600 px-3 py-2 rounded text-sm transition-colors text-left border-l-4 border-purple-500 font-bold text-transparent bg-clip-text bg-gradient-to-r from-red-500 via-yellow-500 to-blue-500" style="text-shadow: 0 0 2px rgba(255,255,255,0.5);">💎 JACKPOT (150 FE)</button>
    </div>

    <div class="flex flex-col gap-2 bg-gray-800 p-2 rounded-lg">
      <h3 class="text-sm font-semibold uppercase text-gray-400">Piggy & Inventory</h3>
      <button @click="spawnLoot(200, 'piggy_test')" class="bg-pink-900 border border-pink-600 hover:bg-pink-800 text-pink-200 px-3 py-2 rounded text-sm transition-colors text-left">🐷 +200 FE (Grow Piggy)</button>
      <button @click="resetPiggy()" class="bg-red-900 border border-red-700 hover:bg-red-800 text-red-200 px-3 py-2 rounded text-sm transition-colors text-left">🔄 Reset Piggy</button>
    </div>

    <div class="flex flex-col gap-2 bg-gray-800 p-2 rounded-lg">
      <h3 class="text-sm font-semibold uppercase text-gray-400">Map & Expenses</h3>
      <button @click="triggerNewMap()" class="bg-blue-900 border border-blue-600 hover:bg-blue-800 px-3 py-2 rounded text-sm transition-colors text-left text-blue-200">🗺️ Start New Map (+ Map Cost Exp)</button>
      <button @click="addCompassRun()" class="bg-orange-900 border border-orange-600 hover:bg-orange-800 px-3 py-2 rounded text-sm transition-colors text-left text-orange-200">🧭 Compass Run (+15 FE Exp)</button>
    </div>

  </div>

  <button
    v-if="!isVisible"
    @click="isVisible = true"
    class="fixed bottom-4 right-4 bg-purple-600 text-white px-4 py-2 rounded-full cursor-pointer shadow-lg hover:bg-purple-500 z-[90] pointer-events-auto border border-purple-400 font-bold"
  >
    🛡️ Open Debug Panel
  </button>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useTrackerStore } from '../stores/trackerStore';

const store = useTrackerStore();
const isVisible = ref(true);

function spawnLoot(price: number, id: string) {
    store.addDrop({
        item_id: id,
        item_name: 'Debug Item (' + price + ' FE)',
        qty: 1,
        price: price,
        timestamp: Date.now()
    });
}

function resetPiggy() {
    store.inventoryValue = 0;
}

function triggerNewMap() {
    store.resetMap();
}

function addCompassRun() {
    store.addExpense({
        id: 'compass_' + Date.now(),
        name: 'Compass Use',
        qty: 1,
        total: 15.0,
        timestamp: Date.now()
    });
}
</script>
