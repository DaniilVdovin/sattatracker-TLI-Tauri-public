<template>
  <div class="w-full h-full relative" style="pointer-events: none; background: rgba(0,0,0,0.002);">

    <Transition name="glow-fade">
      <div
        v-if="screenGlow"
        class="fixed inset-0 pointer-events-none z-[500]"
        :style="{
          boxShadow: screenGlow === 'green'
            ? 'inset 0 0 28px 6px rgba(74,222,128,0.7), inset 0 0 70px 12px rgba(74,222,128,0.22)'
            : 'inset 0 0 28px 6px rgba(168,148,255,0.65), inset 0 0 70px 12px rgba(96,165,250,0.2)',
          borderRadius: 0,
        }"
      />
    </Transition>

    <Transition name="loading-fade">
      <div
        v-if="isLoading"
        class="fixed inset-0 z-[400] flex flex-col items-center justify-end pointer-events-none"
        style="padding-bottom: 48px;"
      >
        <div
          class="flex flex-col items-center gap-3 px-8 py-5 rounded-2xl"
          style="
            background: linear-gradient(160deg, rgba(14,12,22,0.97) 0%, rgba(10,9,18,0.98) 100%);
            border: 1px solid rgba(255,255,255,0.07);
            box-shadow: 0 8px 32px rgba(0,0,0,0.6);
            min-width: 280px;
          "
        >
          <div class="flex items-center gap-2">
            <div class="w-[6px] h-[6px] rounded-full animate-pulse" style="background:#60a5fa; box-shadow:0 0 8px #60a5fa;"></div>
            <span class="text-[11px] font-bold uppercase tracking-[3px]" style="color:rgba(255,255,255,0.4);">TLI Tracker</span>
          </div>

          <div class="text-[13px] font-semibold" style="color:rgba(255,255,255,0.7);">{{ loadingText }}</div>

          <div class="w-full h-[3px] rounded-full overflow-hidden" style="background:rgba(255,255,255,0.07);">
            <div
              class="h-full rounded-full"
              :class="loadingText === 'Ready!' ? 'loading-bar-done' : 'loading-bar-shimmer'"
              style="width:100%;"
            />
          </div>

          <div v-if="loadingLines > 0" class="text-[9px] font-mono" style="color:rgba(255,255,255,0.2);">
            {{ loadingLines.toLocaleString() }} log lines
          </div>
        </div>
      </div>
    </Transition>

    <template v-if="widgetOnlyId">
      <MapTimer        v-if="widgetOnlyId === 'mapTimer'" />
      <MapExpense      v-else-if="widgetOnlyId === 'mapExpense'" />
      <LootFeed        v-else-if="widgetOnlyId === 'lootFeedSession'" mode="session" widgetId="lootFeedSession" />
      <LootFeed        v-else-if="widgetOnlyId === 'lootFeedMap'"     mode="map"     widgetId="lootFeedMap" />
      <ControlPanel    v-else-if="widgetOnlyId === 'controlPanel'" />
      <StatWidget      v-else-if="widgetOnlyId === 'statNetProfitHr'"
        widgetId="statNetProfitHr" title="Profit / Hr"
        :value="store.profitPerHour" unit=" FE/hr" :isPositive="true" />
      <StatWidget      v-else-if="widgetOnlyId === 'statMapProfit'"
        widgetId="statMapProfit" title="Map Profit"
        :value="store.currentMapProfit" unit=" FE" :isPositive="store.currentMapProfit >= 0" />
      <StatWidget      v-else-if="widgetOnlyId === 'statMapsHr'"
        widgetId="statMapsHr" title="Maps / Hr"
        :value="store.mapsPerHour" unit=" /hr" />
      <StatWidget      v-else-if="widgetOnlyId === 'statSessionTime'"
        widgetId="statSessionTime" title="Session Time"
        :value="store.formattedSessionTimer" unit="" />
      <StatWidget      v-else-if="widgetOnlyId === 'statInvValue'"
        widgetId="statInvValue" title="Inventory"
        :value="store.inventoryValue" unit=" FE"
        :clickable="true" @widgetClick="isHistoryOpen = true" />
    </template>

    <template v-else>
      <ExtraEffectsOverlay />
      <PhysicsEffectsOverlay />
      <DebugPanel v-if="showDebug" />

      <BuilderPanel
        :isOpen="isBuilderOpen"
        @close="isBuilderOpen = false"
        @open-settings="isSettingsOpen = true; isBuilderOpen = false; settingsOpenedFromBuilder = true"
        @open-history="isHistoryOpen = true; isBuilderOpen = false"
        @open-map-history="isMapHistoryOpen = true; isBuilderOpen = false"
      />

      <template v-if="!isLoading">

        <ControlPanel />

        <MapTimer />

        <MapExpense />

        <LootFeed mode="session" widgetId="lootFeedSession"
          :defaultPosition="{ x: 50,  y: 200 }" />
        <LootFeed mode="map"     widgetId="lootFeedMap"
          :defaultPosition="{ x: 390, y: 200 }" />

        <StatWidget widgetId="statNetProfitHr"
          title="Profit / Hr"
          :value="store.profitPerHour"    unit=" FE/hr"
          :isPositive="true"
          :defaultPosition="{ x: 50,  y: 500 }" />

        <StatWidget widgetId="statMapProfit"
          title="Map Profit"
          :value="store.currentMapProfit" unit=" FE"
          :isPositive="store.currentMapProfit >= 0"
          :defaultPosition="{ x: 190, y: 500 }" />

        <StatWidget widgetId="statMapsHr"
          title="Maps / Hr"
          :value="store.mapsPerHour"      unit=" /hr"
          :defaultPosition="{ x: 330, y: 500 }" />

        <StatWidget widgetId="statSessionTime"
          title="Session Time"
          :value="store.formattedSessionTimer" unit=""
          :defaultPosition="{ x: 470, y: 500 }" />

        <StatWidget widgetId="statInvValue"
          title="Inventory"
          :value="store.inventoryValue"   unit=" FE"
          :clickable="true"
          @widgetClick="isHistoryOpen = true"
          :defaultPosition="{ x: 610, y: 500 }" />

        <ItemsHistory :isOpen="isHistoryOpen" @close="isHistoryOpen = false" />

        <MapHistory :isOpen="isMapHistoryOpen" @close="isMapHistoryOpen = false" />

      </template>
    </template>

    <SettingsModal
      v-if="!widgetOnlyId"
      :isOpen="isSettingsOpen"
      :showDebugExternal="showDebug"
      @close="onSettingsClose"
      @update:showDebug="showDebug = $event"
    />

    <LowStockWarning v-if="!widgetOnlyId" />


    <WelcomeModal v-if="!widgetOnlyId" />

  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { useTrackerStore } from './stores/trackerStore';
import { useThemeStore } from './stores/themeStore';
import { initAssets } from './utils/assets';
import { checkAndDownloadAssets } from './utils/assetUpdater';
import { updateCursor } from './utils/cursor';

import BuilderPanel         from './components/BuilderPanel.vue';
import StatWidget           from './components/StatWidget.vue';
import MapTimer             from './components/MapTimer.vue';
import MapExpense           from './components/MapExpense.vue';
import ControlPanel         from './components/ControlPanel.vue';
import LootFeed             from './components/LootFeed.vue';
import ItemsHistory         from './components/ItemsHistory.vue';
import MapHistory           from './components/MapHistory.vue';
import ExtraEffectsOverlay  from './components/ExtraEffectsOverlay.vue';
import PhysicsEffectsOverlay from './components/PhysicsEffectsOverlay.vue';
import DebugPanel           from './components/DebugPanel.vue';
import SettingsModal        from './components/SettingsModal.vue';
import WelcomeModal         from './components/WelcomeModal.vue';
import LowStockWarning      from './components/LowStockWarning.vue';

const store      = useTrackerStore();
const themeStore = useThemeStore();

const isSettingsOpen          = ref(false);
const isHistoryOpen           = ref(false);
const isMapHistoryOpen        = ref(false);
const isBuilderOpen           = ref(false);
const settingsOpenedFromBuilder = ref(false);
const showDebug       = ref(false);

const isLoading       = ref(true);
const loadingText     = ref('Connecting...');
const loadingLines    = ref(0);

const isAssetsReady   = ref(false);
const isBackendReady  = ref(false);

function checkReadiness() {
  if (isAssetsReady.value && isBackendReady.value) {
    loadingText.value = 'Ready!';
    setTimeout(() => {
      isLoading.value = false;
      triggerGlow('white');
    }, 600);
  }
}

const screenGlow      = ref<'white' | 'green' | null>(null);
let glowTimeout: ReturnType<typeof setTimeout> | null = null;

function triggerGlow(color: 'white' | 'green') {
  if (glowTimeout) clearTimeout(glowTimeout);
  screenGlow.value = color;
  glowTimeout = setTimeout(() => { screenGlow.value = null; }, 1800);
}

const widgetOnlyId = (() => {
  const search = window.location.search || window.location.hash.split('?')[1] || '';
  return new URLSearchParams(search).get('widgetOnly') ?? '';
})();

let unlistenEditMode: (() => void) | null = null;
let unlistenGame:     (() => void) | null = null;
let timerInterval:    ReturnType<typeof setInterval> | null = null;
let saveInterval:     ReturnType<typeof setInterval> | null = null;

function flushState() {
  if (!widgetOnlyId) void store.saveStateToDisk();
}

function onSettingsClose() {
  isSettingsOpen.value = false;
  if (settingsOpenedFromBuilder.value) {
    isBuilderOpen.value = true;
    settingsOpenedFromBuilder.value = false;
  }
}

const keydownHandler = (e: KeyboardEvent) => {
  if (e.key === 'F8') isBuilderOpen.value = !isBuilderOpen.value;
};

let lastClickThrough = true;
let clickThroughInterval: ReturnType<typeof setInterval> | null = null;

async function updateClickThrough() {
  if (store.isEditMode || isSettingsOpen.value || isHistoryOpen.value || isMapHistoryOpen.value || isBuilderOpen.value) {
    if (lastClickThrough) { lastClickThrough = false; await invoke('set_click_through', { ignore: false }); }
    return;
  }
  try {
    const [sx, sy] = await invoke<[number, number]>('get_cursor_position');
    const dpr = window.devicePixelRatio || 1;
    const lx = sx / dpr, ly = sy / dpr;
    updateCursor(lx, ly);
    const el = document.elementFromPoint(lx, ly);
    const isOverInteractive = !!el && el.closest('[data-interactive]') !== null;
    const want = !isOverInteractive;
    if (want !== lastClickThrough) {
      lastClickThrough = want;
      await invoke('set_click_through', { ignore: want });
    }
  } catch {  }
}

onMounted(async () => {
  await store.loadStateFromDisk();

  unlistenGame = await listen<any>('game_event', (event) => {
    const e = event.payload;
    switch (e.type) {
      case 'LootDrop':
        store.addDrop({
          item_id:   e.item_id,
          item_name: e.item_name,
          qty:       e.qty,
          price:     e.price,
          timestamp: Date.now(),
        });
        break;
      case 'ZoneChange':
        store.onZoneChange(e.level_type, e.level_id, e.is_hub ?? false);
        break;
      case 'MapStart':
        store.resetMap(true, { levelType: e.level_type, levelId: e.level_id });
        break;
      case 'Expense':
        store.addExpense({
          id:        e.item_id,
          name:      e.item_name,
          qty:       e.qty,
          total:     e.total,
          timestamp: Date.now(),
        });
        store.trackConsumable(e.item_id);
        break;
      case 'LoadingStart':
        isLoading.value   = true;
        if (!loadingText.value.includes('Downloading') && !loadingText.value.includes('Extract')) {
          loadingText.value = 'Initializing...';
        }
        loadingLines.value = e.total_lines ?? 0;
        break;
      case 'LoadingDone':
        isBackendReady.value = true;
        checkReadiness();
        break;
      case 'InventoryRefreshed':
        if (e.items && e.items.length > 0) {
          store.inventoryItems = e.items;
          store.checkLowStock(e.items);
        }
        if (e.value > 0) {
          store.inventoryValue = Math.floor(e.value);
        }
        triggerGlow('green');
        break;
    }
  });

  await checkAndDownloadAssets((text) => {
    loadingText.value = text;
  });
  await initAssets();

  isAssetsReady.value = true;
  checkReadiness();

  store.loadConfigFromStorage();
  themeStore.loadFromStorage();

  unlistenEditMode = await listen('toggle_edit_mode', () => {
    isBuilderOpen.value = !isBuilderOpen.value;
  });

  timerInterval = setInterval(() => {
    store.tickTimers();
  }, 1000);

  clickThroughInterval = setInterval(updateClickThrough, 60);

  if (!widgetOnlyId) {
    saveInterval = setInterval(() => {
      if (store.stateDirty) void store.saveStateToDisk();
    }, 10000);
    window.addEventListener('beforeunload', flushState);
  }

  window.addEventListener('keydown', keydownHandler);
});

onUnmounted(() => {
  if (unlistenEditMode)      unlistenEditMode();
  if (unlistenGame)          unlistenGame();
  if (timerInterval)         clearInterval(timerInterval);
  if (clickThroughInterval)  clearInterval(clickThroughInterval);
  if (saveInterval)          clearInterval(saveInterval);
  window.removeEventListener('beforeunload', flushState);
  window.removeEventListener('keydown', keydownHandler);
});
</script>

<style>
body {
  margin: 0;
  padding: 0;
  background-color: transparent !important;
}
#app {
  background-color: transparent !important;
  width: 100vw;
  height: 100vh;
}
</style>
