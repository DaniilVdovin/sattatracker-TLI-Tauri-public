import { defineStore } from 'pinia';

export interface InventorySnapshotItem {
  item_id:   string;
  item_name: string;
  qty:       number;
  price:     number;
}

export interface LootItem {
  item_id: string;
  item_name: string;
  qty: number;
  price: number;
  timestamp: number;
}

export interface ExpenseItem {
  id: string;
  name: string;
  qty: number;
  total: number;
  timestamp: number;
}

export const useTrackerStore = defineStore('tracker', {
  state: () => ({
    minPriceThreshold: 0.1,
    mapCost: 0,
    applyTax: false,
    ignoreAuction: false,

    dropFxStyle: 'modern' as 'classic' | 'modern' | 'none',
    extraFxStyle: 'piggy' as 'casino' | 'piggy' | 'web' | 'chain' | 'orbital' | 'bubbles' | 'swarm' | 'rings' | 'slash' | 'shockwave' | 'confetti' | 'lightning' | 'none',

    widgetOpacity: 0.9,

    fxDuration: 4.0,

    fxX: 50,
    fxY: 50,

    fxScale: 1.0,

    pauseTimerOnHub: true,

    isEditMode: false,

    piggyCombo: 0,

    totalProfit: 0,
    sessionLootCount: 0,
    sessionDrops: [] as LootItem[],
    sessionMapCount: 0,

    inventoryValue: 0,
    inventoryItems: [] as InventorySnapshotItem[],

    currentMapProfit: 0,
    currentMapDrops: [] as LootItem[],
    currentZone: 'Town',

    mapExpenses: [] as ExpenseItem[],

    sessionTimeSec: 0,
    mapTimeSec: 0,
    isTimerRunning: true,

    latestDrop: null as LootItem | null,

    blockedItemIds: [] as string[],

    trackedConsumableIds: [] as string[],

    lowStockWarnings: [] as Array<{ item_id: string; item_name: string; qty: number }>,
    lowStockEnabled: true,

    piggyTier: 0,

    widgetPositions: {
      controlPanel: { x: 400, y: 100, show: true, w: undefined as number|undefined, h: undefined as number|undefined },
      piggyBank: { x: 700, y: 200, show: true, w: 350 as number|undefined, h: 350 as number|undefined },

      statNetProfitHr: { x: 50, y: 500, show: true, w: undefined as number|undefined, h: undefined as number|undefined },
      statMapProfit: { x: 180, y: 500, show: true, w: undefined as number|undefined, h: undefined as number|undefined },
      statMapsHr: { x: 310, y: 500, show: true, w: undefined as number|undefined, h: undefined as number|undefined },
      statSessionTime: { x: 440, y: 500, show: true, w: undefined as number|undefined, h: undefined as number|undefined },
      statInvValue: { x: 570, y: 500, show: true, w: undefined as number|undefined, h: undefined as number|undefined },

      sessionStats: { x: -999, y: 150, show: false, w: undefined as number|undefined, h: undefined as number|undefined },
      mapStats: { x: -999, y: 250, show: false, w: undefined as number|undefined, h: undefined as number|undefined },

      lootFeedSession: { x: 50, y: 200, show: true, w: undefined as number|undefined, h: undefined as number|undefined },
      lootFeedMap: { x: 380, y: 200, show: true, w: undefined as number|undefined, h: undefined as number|undefined },

      mapTimer: { x: 800, y: 50, show: true, w: undefined as number|undefined, h: undefined as number|undefined },
      mapExpense: { x: 50, y: 400, show: true, w: undefined as number|undefined, h: undefined as number|undefined },
      itemsHistory: { x: 200, y: 100, show: false, w: 800, h: 600 }
    }
  }),

  actions: {
    trackConsumable(itemId: string) {
      if (!this.trackedConsumableIds.includes(itemId)) {
        this.trackedConsumableIds.push(itemId);
      }
    },

    checkLowStock(items: InventorySnapshotItem[]) {
      const itemMap = new Map(items.map(i => [i.item_id, i]));
      this.lowStockWarnings = this.trackedConsumableIds
        .map(id => itemMap.get(id))
        .filter((item): item is InventorySnapshotItem => !!item && item.qty > 0 && item.qty <= 3)
        .map(item => ({ item_id: item.item_id, item_name: item.item_name, qty: item.qty }));
    },

    blockItem(itemId: string) {
      if (!this.blockedItemIds.includes(itemId)) {
        this.blockedItemIds.push(itemId);
        this.saveConfigToStorage();
      }
    },

    unblockItem(itemId: string) {
      this.blockedItemIds = this.blockedItemIds.filter(id => id !== itemId);
      this.saveConfigToStorage();
    },

    isBlocked(itemId: string): boolean {
      return this.blockedItemIds.includes(itemId);
    },

    toggleWidget(widgetId: string) {
      const w = (this.widgetPositions as any)[widgetId];
      if (w) w.show = !w.show;
    },

    tickTimers() {
      if (this.isTimerRunning) {
        this.sessionTimeSec++;
        this.mapTimeSec++;
      }
    },

    pauseSession() {
      this.isTimerRunning = false;
    },

    resumeSession() {
      this.isTimerRunning = true;
    },

    onZoneChange(_levelType: number, levelId: number, isHub: boolean) {
      if (isHub) {
        this.currentZone = 'Town';
        if (this.pauseTimerOnHub) this.pauseSession();
      } else {
        this.currentZone = `Map ${levelId}`;
      }
    },

    addDrop(item: LootItem) {
      if (item.qty <= 0) return;

      let effectivePrice = item.price;
      if (this.applyTax && item.item_id !== '100300') {
        effectivePrice = item.price * 0.875;
      }

      const totalValue = Math.round(effectivePrice * item.qty * 100) / 100;

      if (totalValue < this.minPriceThreshold) return;

      const taxedItem: LootItem = { ...item, price: effectivePrice };

      this.totalProfit      = Math.round((this.totalProfit      + totalValue) * 100) / 100;
      this.currentMapProfit = Math.round((this.currentMapProfit + totalValue) * 100) / 100;
      this.sessionLootCount += item.qty;
      this.inventoryValue   = Math.round((this.inventoryValue   + totalValue) * 100) / 100;

      const existingSession = this.sessionDrops.find(d => d.item_id === taxedItem.item_id);
      if (existingSession) {
        const prevTotal = existingSession.price * existingSession.qty;
        existingSession.qty += taxedItem.qty;
        existingSession.price = (prevTotal + totalValue) / existingSession.qty;
      } else {
        this.sessionDrops.push(taxedItem);
      }

      const existingMap = this.currentMapDrops.find(d => d.item_id === taxedItem.item_id);
      if (existingMap) {
        const prevTotal = existingMap.price * existingMap.qty;
        existingMap.qty += taxedItem.qty;
        existingMap.price = (prevTotal + totalValue) / existingMap.qty;
      } else {
        this.currentMapDrops.push(taxedItem);
      }

      this.latestDrop = taxedItem;
    },

    addExpense(expense: ExpenseItem) {
        const existing = this.mapExpenses.find(e => e.id === expense.id);
        if (existing) {
            existing.qty   += expense.qty;
            existing.total  = Math.round((existing.total + expense.total) * 100) / 100;
        } else {
            this.mapExpenses.push(expense);
        }
        this.currentMapProfit = Math.round((this.currentMapProfit - expense.total) * 100) / 100;
        this.totalProfit      = Math.round((this.totalProfit      - expense.total) * 100) / 100;
    },

    setZone(zone: string) {
      this.currentZone = zone;
    },

    resetMap(fromMapStart = false) {
      this.sessionMapCount++;
      this.mapExpenses = [];
      this.currentMapProfit = 0;
      this.mapTimeSec = 0;
      this.currentMapDrops = [];
      if (fromMapStart) this.resumeSession();

      if (this.mapCost > 0) {
        this.addExpense({
            id: 'map_cost',
            name: 'Map Activation',
            qty: 1,
            total: this.mapCost,
            timestamp: Date.now()
        });
      }
    },

    resetSession() {
      this.totalProfit = -this.mapCost;
      this.sessionTimeSec = 0;
      this.sessionDrops = [];
      this.sessionLootCount = 0;
      this.sessionMapCount = 0;
      this.inventoryValue = 0;
      this.resetMap();
    },

    updatePosition(widgetId: keyof typeof this.widgetPositions, x: number, y: number, w?: number, h?: number) {
      if (this.widgetPositions[widgetId]) {
        this.widgetPositions[widgetId].x = x;
        this.widgetPositions[widgetId].y = y;
        if (w !== undefined) (this.widgetPositions[widgetId] as any).w = w;
        if (h !== undefined) (this.widgetPositions[widgetId] as any).h = h;
      }
    },

    saveConfigToStorage() {
      const stateToSave = {
        minPriceThreshold: this.minPriceThreshold,
        mapCost: this.mapCost,
        applyTax: this.applyTax,
        ignoreAuction: this.ignoreAuction,
        dropFxStyle: this.dropFxStyle,
        extraFxStyle: this.extraFxStyle,
        widgetOpacity: this.widgetOpacity,
        fxDuration: this.fxDuration,
        fxX: this.fxX,
        fxY: this.fxY,
        fxScale: this.fxScale,
        pauseTimerOnHub: this.pauseTimerOnHub,
        widgetPositions: this.widgetPositions,
        blockedItemIds: this.blockedItemIds,
        lowStockEnabled: this.lowStockEnabled
      };
      localStorage.setItem('tracker_config_v2', JSON.stringify(stateToSave));
    },

    loadConfigFromStorage() {
      const stored = localStorage.getItem('tracker_config_v2');
      if (stored) {
        const parsed = JSON.parse(stored);
        if (parsed.minPriceThreshold !== undefined) this.minPriceThreshold = parsed.minPriceThreshold;
        if (parsed.mapCost !== undefined) this.mapCost = parsed.mapCost;
        if (parsed.applyTax !== undefined) this.applyTax = parsed.applyTax;
        if (parsed.ignoreAuction !== undefined) this.ignoreAuction = parsed.ignoreAuction;
        if (parsed.dropFxStyle) this.dropFxStyle = parsed.dropFxStyle;
        if (parsed.extraFxStyle) this.extraFxStyle = parsed.extraFxStyle;
        if (typeof parsed.widgetOpacity === 'number') this.widgetOpacity = parsed.widgetOpacity;
        if (typeof parsed.fxDuration === 'number') this.fxDuration = parsed.fxDuration;
        if (typeof parsed.fxX === 'number') this.fxX = parsed.fxX;
        if (typeof parsed.fxY === 'number') this.fxY = parsed.fxY;
        if (typeof parsed.fxScale === 'number') this.fxScale = parsed.fxScale;
        if (typeof parsed.pauseTimerOnHub === 'boolean') this.pauseTimerOnHub = parsed.pauseTimerOnHub;
        if (parsed.widgetPositions) {
          this.widgetPositions = { ...this.widgetPositions, ...parsed.widgetPositions };
        }
        if (parsed.blockedItemIds) this.blockedItemIds = parsed.blockedItemIds;
        if (typeof parsed.lowStockEnabled === 'boolean') this.lowStockEnabled = parsed.lowStockEnabled;
      }
    }
  },

  getters: {
    profitPerHour: (state) => {
      const hours = Math.max(state.sessionTimeSec, 1) / 3600;
      return (state.totalProfit / hours).toFixed(1);
    },
    mapsPerHour: (state) => {
      const hours = Math.max(state.sessionTimeSec, 1) / 3600;
      return (state.sessionMapCount / hours).toFixed(1);
    },
    formattedMapTimer: (state) => {
      const m = Math.floor(state.mapTimeSec / 60).toString().padStart(2, '0');
      const s = (state.mapTimeSec % 60).toString().padStart(2, '0');
      return `${m}:${s}`;
    },
    formattedSessionTimer: (state) => {
      const h = Math.floor(state.sessionTimeSec / 3600);
      const m = Math.floor((state.sessionTimeSec % 3600) / 60).toString().padStart(2, '0');
      const s = (state.sessionTimeSec % 60).toString().padStart(2, '0');
      return h > 0 ? `${h}:${m}:${s}` : `${m}:${s}`;
    }
  }
});
