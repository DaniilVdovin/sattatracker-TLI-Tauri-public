<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-[150] flex items-center justify-center pointer-events-auto"
    style="background: rgba(0,0,0,0.8);"
    @click.self="$emit('close')"
  >
    <div
      class="flex overflow-hidden select-none"
      style="
        width: 940px; height: 620px;
        background: linear-gradient(160deg, rgba(14,12,22,0.98) 0%, rgba(10,9,18,0.99) 100%);
        border: 1px solid rgba(255,255,255,0.09);
        border-radius: 18px;
        box-shadow: 0 24px 80px rgba(0,0,0,0.85), inset 0 1px 0 rgba(255,255,255,0.06);
      "
    >
      <div class="flex flex-col shrink-0" style="width: 340px; border-right: 1px solid rgba(255,255,255,0.06);">
        <div class="px-4 pt-4 pb-3" style="border-bottom: 1px solid rgba(255,255,255,0.05);">
          <div class="flex items-center justify-between">
            <div class="text-[11px] font-black uppercase tracking-[3px]" style="color: rgba(255,255,255,0.55);">Map History</div>
            <button
              class="text-[11px] font-bold px-3 py-[3px] rounded-lg"
              style="background: rgba(248,113,113,0.1); border: 1px solid rgba(248,113,113,0.25); color: #f87171;"
              @click="$emit('close')"
            >Close</button>
          </div>

          <select
            v-model="scope"
            class="w-full mt-3 text-[12px] px-2 py-[6px] rounded-lg outline-none"
            style="background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.1); color: rgba(255,255,255,0.8);"
          >
            <option value="current">Current session</option>
            <option value="all">All time</option>
            <option v-for="s in sessionOptions" :key="s.id" :value="s.id">{{ s.label }}</option>
          </select>

          <div class="grid grid-cols-3 gap-2 mt-3">
            <div v-for="cell in summaryCells" :key="cell.label">
              <div class="text-[9px] uppercase tracking-[2px]" style="color: rgba(255,255,255,0.25);">{{ cell.label }}</div>
              <div class="text-[13px] font-bold font-mono" :style="{ color: cell.color }">{{ cell.value }}</div>
            </div>
          </div>
        </div>

        <div class="flex-1 overflow-y-auto custom-scroll px-3 py-3" @contextmenu.prevent>
          <div v-if="rows.length === 0" class="flex items-center justify-center h-full">
            <div class="text-center">
              <div class="text-[32px] mb-2 opacity-30">🗺️</div>
              <div class="text-[13px]" style="color: rgba(255,255,255,0.25);">No maps recorded yet</div>
            </div>
          </div>

          <div v-else class="flex flex-col gap-[4px]">
            <div
              v-for="r in rows"
              :key="r.id"
              class="flex items-stretch rounded-xl cursor-pointer overflow-hidden"
              :style="{
                background: selectedId === r.id ? 'rgba(96,165,250,0.12)' : 'rgba(255,255,255,0.03)',
                border: selectedId === r.id ? '1px solid rgba(96,165,250,0.4)' : '1px solid rgba(255,255,255,0.05)',
              }"
              @click="selectedId = r.id"
            >
              <div class="w-[3px] shrink-0" :style="{ background: r.profit >= 0 ? '#4ade80' : '#f87171' }" />
              <div class="flex-1 min-w-0 px-3 py-[7px]">
                <div class="flex items-center justify-between gap-2">
                  <div class="text-[12px] font-semibold truncate" style="color: rgba(255,255,255,0.85);">
                    <span v-if="r.live" class="mr-1" style="color:#60a5fa;">●</span>{{ mapTitle(r) }}
                  </div>
                  <div class="text-[13px] font-bold font-mono shrink-0" :style="{ color: r.profit >= 0 ? '#4ade80' : '#f87171' }">
                    {{ fmtFE(r.profit) }}
                  </div>
                </div>
                <div class="flex items-center justify-between text-[10px] mt-[2px]" style="color: rgba(255,255,255,0.3);">
                  <span>{{ fmtWhen(r.startedAt) }} · {{ fmtDur(r.durationSec) }}</span>
                  <span>{{ r.lootCount }} items{{ r.expenseValue > 0 ? ' · −' + r.expenseValue.toFixed(0) : '' }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="flex-1 flex flex-col min-w-0">
        <div v-if="!selected" class="flex-1 flex items-center justify-center">
          <div class="text-[13px]" style="color: rgba(255,255,255,0.25);">Select a map to see details</div>
        </div>

        <template v-else>
          <div class="px-5 pt-4 pb-3" style="border-bottom: 1px solid rgba(255,255,255,0.06);">
            <div class="flex items-baseline gap-3">
              <div class="text-[16px] font-bold" style="color: rgba(255,255,255,0.85);">{{ mapTitle(selected) }}</div>
              <div v-if="selected.live" class="text-[10px] font-bold px-2 py-[1px] rounded-full" style="background: rgba(96,165,250,0.15); color:#60a5fa; border:1px solid rgba(96,165,250,0.3);">IN PROGRESS</div>
            </div>
            <div class="text-[11px] mt-[2px]" style="color: rgba(255,255,255,0.3);">{{ fmtFull(selected.startedAt) }}</div>

            <div class="grid grid-cols-5 gap-3 mt-3">
              <div v-for="cell in detailCells" :key="cell.label">
                <div class="text-[9px] uppercase tracking-[2px]" style="color: rgba(255,255,255,0.25);">{{ cell.label }}</div>
                <div class="text-[14px] font-bold font-mono" :style="{ color: cell.color }">{{ cell.value }}</div>
              </div>
            </div>
          </div>

          <div class="flex-1 overflow-y-auto custom-scroll px-5 py-4" @contextmenu.prevent>
            <div class="text-[10px] uppercase tracking-[2px] mb-2" style="color: rgba(255,255,255,0.3);">Net profit over time</div>
            <div class="rounded-xl px-2 py-2" style="background: rgba(255,255,255,0.025); border: 1px solid rgba(255,255,255,0.05);">
              <svg
                :viewBox="`0 0 ${CW} ${CH}`"
                class="w-full block"
                style="height: 170px;"
                @mousemove="onChartMove"
                @mouseleave="hover = null"
              >
                <line
                  v-for="g in chart.grid"
                  :key="g.y"
                  :x1="PAD_L" :x2="CW - PAD_R" :y1="g.y" :y2="g.y"
                  :stroke="g.zero ? 'rgba(255,255,255,0.28)' : 'rgba(255,255,255,0.07)'"
                  :stroke-dasharray="g.zero ? '4 3' : ''"
                />
                <text
                  v-for="g in chart.grid"
                  :key="'l' + g.y"
                  :x="PAD_L - 6" :y="g.y + 3"
                  text-anchor="end" font-size="9" fill="rgba(255,255,255,0.35)"
                >{{ g.label }}</text>
                <text
                  v-for="tk in chart.ticks"
                  :key="'t' + tk.x"
                  :x="tk.x" :y="CH - 5"
                  :text-anchor="tk.anchor" font-size="9" fill="rgba(255,255,255,0.35)"
                >{{ tk.label }}</text>
                <path :d="chart.area" :fill="chart.color" fill-opacity="0.12" />
                <path :d="chart.line" fill="none" :stroke="chart.color" stroke-width="2" stroke-linejoin="round" />
                <template v-if="hover">
                  <line :x1="hover.x" :x2="hover.x" :y1="PAD_T" :y2="CH - PAD_B" stroke="rgba(255,255,255,0.3)" />
                  <circle :cx="hover.x" :cy="hover.y" r="3.5" :fill="chart.color" />
                  <text
                    :x="hover.x > CW / 2 ? hover.x - 8 : hover.x + 8" :y="PAD_T + 10"
                    :text-anchor="hover.x > CW / 2 ? 'end' : 'start'"
                    font-size="10" fill="rgba(255,255,255,0.85)"
                  >{{ hover.label }}</text>
                </template>
              </svg>
            </div>

            <div class="text-[10px] uppercase tracking-[2px] mt-5 mb-2" style="color: rgba(255,255,255,0.3);">
              Loot ({{ sortedDrops.length }})
            </div>
            <div v-if="sortedDrops.length === 0" class="text-[12px]" style="color: rgba(255,255,255,0.25);">No loot recorded</div>
            <div v-else class="flex flex-col gap-[4px]">
              <div
                v-for="d in sortedDrops"
                :key="d.item_id"
                class="flex items-center gap-3 px-3 py-[6px] rounded-xl"
                style="background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.05);"
              >
                <div
                  class="w-8 h-8 rounded-lg shrink-0 flex items-center justify-center overflow-hidden"
                  style="background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.08);"
                >
                  <img
                    :src="getIcon(d.item_id)"
                    @error="($event.target as HTMLImageElement).style.display='none'"
                    class="w-6 h-6 object-contain"
                  />
                </div>
                <div class="flex-1 min-w-0 text-[12px] font-semibold truncate" style="color: rgba(255,255,255,0.85);">{{ d.item_name || 'Item ' + d.item_id }}</div>
                <div class="text-right shrink-0">
                  <div class="text-[12px] font-bold" style="color:#fbbf24;">{{ (d.price * d.qty).toFixed(1) }} FE</div>
                  <div class="text-[10px]" style="color: rgba(255,255,255,0.3);">x{{ d.qty }} @ {{ d.price.toFixed(2) }}</div>
                </div>
              </div>
            </div>

            <template v-if="selected.expenses.length > 0">
              <div class="text-[10px] uppercase tracking-[2px] mt-5 mb-2" style="color: rgba(255,255,255,0.3);">Expenses</div>
              <div class="flex flex-col gap-[4px]">
                <div
                  v-for="e in selected.expenses"
                  :key="e.id"
                  class="flex items-center gap-3 px-3 py-[6px] rounded-xl"
                  style="background: rgba(248,113,113,0.05); border: 1px solid rgba(248,113,113,0.12);"
                >
                  <div class="flex-1 min-w-0 text-[12px] font-semibold truncate" style="color: rgba(255,255,255,0.75);">{{ e.name || e.id }}</div>
                  <div class="text-[10px]" style="color: rgba(255,255,255,0.3);">x{{ e.qty }}</div>
                  <div class="text-[12px] font-bold" style="color:#f87171;">−{{ e.total.toFixed(1) }} FE</div>
                </div>
              </div>
            </template>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useTrackerStore } from '../stores/trackerStore';
import { getIcon } from '../utils/assets';
import type { MapRecord } from '../types/history';

const props = defineProps<{ isOpen: boolean }>();
defineEmits(['close']);

const store = useTrackerStore();

type Row = MapRecord & { live?: boolean };

const scope = ref<string>('current');
const selectedId = ref<string>('');

const CW = 600;
const CH = 170;
const PAD_L = 46;
const PAD_R = 12;
const PAD_T = 10;
const PAD_B = 22;

function fmtDur(sec: number): string {
  const s = Math.max(0, Math.round(sec));
  const m = Math.floor(s / 60);
  const h = Math.floor(m / 60);
  const ss = String(s % 60).padStart(2, '0');
  if (h > 0) return `${h}:${String(m % 60).padStart(2, '0')}:${ss}`;
  return `${String(m).padStart(2, '0')}:${ss}`;
}

function fmtFE(v: number): string {
  const sign = v > 0 ? '+' : v < 0 ? '−' : '';
  const a = Math.abs(v);
  const body = a >= 10000 ? (a / 1000).toFixed(1) + 'k' : a >= 100 ? a.toFixed(0) : a.toFixed(1);
  return `${sign}${body}`;
}

function fmtTime(ms: number): string {
  return new Date(ms).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

function fmtWhen(ms: number): string {
  const d = new Date(ms);
  const today = new Date();
  const sameDay = d.toDateString() === today.toDateString();
  return sameDay ? fmtTime(ms) : `${d.toLocaleDateString([], { day: '2-digit', month: '2-digit' })} ${fmtTime(ms)}`;
}

function fmtFull(ms: number): string {
  return new Date(ms).toLocaleString([], {
    day: '2-digit', month: '2-digit', year: 'numeric', hour: '2-digit', minute: '2-digit', second: '2-digit',
  });
}

function mapTitle(r: Row): string {
  return `Map ${r.levelId}`;
}

const liveRow = computed<Row | null>(() => {
  const meta = store.currentMapMeta;
  if (!meta) return null;
  const lootValue = store.currentMapDrops.reduce((a, d) => a + d.price * d.qty, 0);
  const expenseValue = store.mapExpenses.reduce((a, e) => a + e.total, 0);
  return {
    id: 'live',
    live: true,
    sessionId: store.sessionId,
    levelType: meta.levelType,
    levelId: meta.levelId,
    startedAt: meta.startedAt,
    endedAt: Date.now(),
    durationSec: store.mapTimeSec,
    profit: store.currentMapProfit,
    lootValue,
    expenseValue,
    lootCount: store.currentMapDrops.reduce((a, d) => a + d.qty, 0),
    drops: store.currentMapDrops.map(d => ({ item_id: d.item_id, item_name: d.item_name, qty: d.qty, price: d.price })),
    expenses: store.mapExpenses.map(e => ({ id: e.id, name: e.name, qty: e.qty, total: e.total })),
    timeline: store.mapTimeline.length ? [...store.mapTimeline] : [{ t: 0, v: 0 }],
  };
});

const sessionOptions = computed(() =>
  [...store.archivedSessions]
    .sort((a, b) => b.startedAt - a.startedAt)
    .map(s => ({
      id: s.id,
      label: `${fmtWhen(s.startedAt)} · ${s.maps} maps · ${fmtFE(s.profit)} FE`,
    })),
);

const rows = computed<Row[]>(() => {
  let list: Row[];
  if (scope.value === 'all') list = [...store.mapHistory];
  else if (scope.value === 'current') list = store.mapHistory.filter(m => m.sessionId === store.sessionId);
  else list = store.mapHistory.filter(m => m.sessionId === scope.value);
  list.sort((a, b) => b.startedAt - a.startedAt);
  const live = liveRow.value;
  if (live && (scope.value === 'current' || scope.value === 'all')) list.unshift(live);
  return list;
});

const selected = computed<Row | null>(() => rows.value.find(r => r.id === selectedId.value) ?? null);

watch(
  [() => props.isOpen, scope],
  () => {
    if (!rows.value.find(r => r.id === selectedId.value)) selectedId.value = rows.value[0]?.id ?? '';
  },
  { immediate: true },
);

watch(rows, (list) => {
  if (!list.find(r => r.id === selectedId.value)) selectedId.value = list[0]?.id ?? '';
});

const summaryCells = computed(() => {
  const list = rows.value;
  const profit = list.reduce((a, r) => a + r.profit, 0);
  const secs = list.reduce((a, r) => a + r.durationSec, 0);
  const perHour = secs > 0 ? (profit / secs) * 3600 : 0;
  return [
    { label: 'Maps', value: String(list.length), color: 'rgba(255,255,255,0.8)' },
    { label: 'Profit', value: fmtFE(profit), color: profit >= 0 ? '#4ade80' : '#f87171' },
    { label: 'Per hour', value: fmtFE(perHour), color: perHour >= 0 ? '#4ade80' : '#f87171' },
  ];
});

const detailCells = computed(() => {
  const r = selected.value;
  if (!r) return [];
  const perHour = r.durationSec > 0 ? (r.profit / r.durationSec) * 3600 : 0;
  return [
    { label: 'Time', value: fmtDur(r.durationSec), color: 'rgba(255,255,255,0.85)' },
    { label: 'Profit', value: fmtFE(r.profit), color: r.profit >= 0 ? '#4ade80' : '#f87171' },
    { label: 'Per hour', value: fmtFE(perHour), color: perHour >= 0 ? '#4ade80' : '#f87171' },
    { label: 'Loot', value: r.lootValue.toFixed(0), color: '#fbbf24' },
    { label: 'Cost', value: r.expenseValue > 0 ? '−' + r.expenseValue.toFixed(0) : '0', color: r.expenseValue > 0 ? '#f87171' : 'rgba(255,255,255,0.5)' },
  ];
});

const sortedDrops = computed(() =>
  selected.value ? [...selected.value.drops].sort((a, b) => b.price * b.qty - a.price * a.qty) : [],
);

interface ChartPoint { x: number; y: number; t: number; v: number }

const chart = computed(() => {
  const r = selected.value;
  const empty = { line: '', area: '', color: '#60a5fa', grid: [] as Array<{ y: number; label: string; zero: boolean }>, ticks: [] as Array<{ x: number; label: string; anchor: string }>, points: [] as ChartPoint[] };
  if (!r) return empty;

  const raw = r.timeline.length ? r.timeline : [{ t: 0, v: 0 }];
  const tMax = Math.max(r.durationSec, raw[raw.length - 1].t, 1);
  const vals = raw.map(p => p.v).concat([0, r.profit]);
  let vMin = Math.min(...vals);
  let vMax = Math.max(...vals);
  if (vMax - vMin < 1) { vMax += 1; vMin -= 1; }
  const margin = (vMax - vMin) * 0.08;
  vMin -= margin;
  vMax += margin;

  const w = CW - PAD_L - PAD_R;
  const h = CH - PAD_T - PAD_B;
  const sx = (t: number) => PAD_L + (t / tMax) * w;
  const sy = (v: number) => PAD_T + (1 - (v - vMin) / (vMax - vMin)) * h;

  const pts: ChartPoint[] = raw.map(p => ({ x: sx(p.t), y: sy(p.v), t: p.t, v: p.v }));
  const lastV = raw[raw.length - 1].v;
  pts.push({ x: sx(tMax), y: sy(lastV), t: tMax, v: lastV });

  let line = `M ${pts[0].x.toFixed(1)} ${pts[0].y.toFixed(1)}`;
  for (let i = 1; i < pts.length; i++) {
    line += ` H ${pts[i].x.toFixed(1)} V ${pts[i].y.toFixed(1)}`;
  }
  const zeroY = sy(0);
  const area = `${line} V ${zeroY.toFixed(1)} H ${pts[0].x.toFixed(1)} Z`;

  const color = lastV >= 0 ? '#4ade80' : '#f87171';
  const levels = [vMax - margin, 0, vMin + margin];
  const grid = levels.map((v, i) => ({ y: sy(v), label: fmtFE(v).replace('+', ''), zero: i === 1 }));
  const ticks = [
    { x: sx(0), label: '00:00', anchor: 'start' },
    { x: sx(tMax / 2), label: fmtDur(tMax / 2), anchor: 'middle' },
    { x: sx(tMax), label: fmtDur(tMax), anchor: 'end' },
  ];
  return { line, area, color, grid, ticks, points: pts };
});

const hover = ref<{ x: number; y: number; label: string } | null>(null);

function onChartMove(e: MouseEvent) {
  const svg = e.currentTarget as SVGSVGElement;
  const rect = svg.getBoundingClientRect();
  if (rect.width === 0) return;
  const px = ((e.clientX - rect.left) / rect.width) * CW;
  const pts = chart.value.points;
  if (!pts.length) return;
  let best = pts[0];
  for (const p of pts) {
    if (Math.abs(p.x - px) < Math.abs(best.x - px)) best = p;
  }
  hover.value = { x: best.x, y: best.y, label: `${fmtDur(best.t)}  ${fmtFE(best.v)} FE` };
}
</script>
