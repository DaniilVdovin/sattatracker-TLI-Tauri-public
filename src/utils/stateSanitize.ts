import {
  MAX_ARCHIVED_SESSIONS,
  MAX_MAP_HISTORY,
  MAX_RECORD_DROPS,
  MAX_TIMELINE_POINTS,
  type MapRecord,
  type RecordDrop,
  type RecordExpense,
  type SessionRecord,
  type TimelinePoint,
} from '../types/history';

type Obj = Record<string, unknown>;

const isObj = (v: unknown): v is Obj => typeof v === 'object' && v !== null && !Array.isArray(v);

export function num(v: unknown, min = -1e9, max = 1e9, fallback = 0): number {
  if (typeof v !== 'number' || !Number.isFinite(v)) return fallback;
  return Math.min(max, Math.max(min, v));
}

export function text(v: unknown, max = 120): string {
  return typeof v === 'string' ? v.slice(0, max) : '';
}

export function itemId(v: unknown): string {
  return typeof v === 'string' && /^\d{1,12}$/.test(v) ? v : '';
}

function safeId(v: unknown): string {
  return typeof v === 'string' && /^[0-9A-Za-z_-]{1,64}$/.test(v) ? v : '';
}

function list(v: unknown, max: number): unknown[] {
  return Array.isArray(v) ? v.slice(0, max) : [];
}

export function sanitizeDrops(v: unknown, max = MAX_RECORD_DROPS): RecordDrop[] {
  const out: RecordDrop[] = [];
  for (const d of list(v, max)) {
    if (!isObj(d)) continue;
    const id = itemId(d.item_id);
    if (!id) continue;
    out.push({
      item_id: id,
      item_name: text(d.item_name, 120),
      qty: num(d.qty, 0, 1e9),
      price: num(d.price, 0, 1e8),
    });
  }
  return out;
}

export function sanitizeExpenses(v: unknown, max = 60): RecordExpense[] {
  const out: RecordExpense[] = [];
  for (const e of list(v, max)) {
    if (!isObj(e)) continue;
    const id = text(e.id, 64);
    if (!id) continue;
    out.push({
      id,
      name: text(e.name, 120),
      qty: num(e.qty, 0, 1e9),
      total: num(e.total, 0, 1e9),
    });
  }
  return out;
}

export function sanitizeTimeline(v: unknown): TimelinePoint[] {
  const out: TimelinePoint[] = [];
  for (const p of list(v, MAX_TIMELINE_POINTS * 2)) {
    if (!isObj(p)) continue;
    out.push({ t: num(p.t, 0, 1e7), v: num(p.v) });
  }
  return out;
}

export function sanitizeMapRecord(v: unknown): MapRecord | null {
  if (!isObj(v)) return null;
  const id = safeId(v.id);
  const sessionId = safeId(v.sessionId);
  if (!id || !sessionId) return null;
  return {
    id,
    sessionId,
    levelType: num(v.levelType, 0, 1000),
    levelId: num(v.levelId, 0, 1e12),
    startedAt: num(v.startedAt, 0, 1e14),
    endedAt: num(v.endedAt, 0, 1e14),
    durationSec: num(v.durationSec, 0, 1e7),
    profit: num(v.profit),
    lootValue: num(v.lootValue, 0),
    expenseValue: num(v.expenseValue, 0),
    lootCount: num(v.lootCount, 0),
    drops: sanitizeDrops(v.drops),
    expenses: sanitizeExpenses(v.expenses),
    timeline: sanitizeTimeline(v.timeline),
  };
}

export function sanitizeSessionRecord(v: unknown): SessionRecord | null {
  if (!isObj(v)) return null;
  const id = safeId(v.id);
  if (!id) return null;
  return {
    id,
    startedAt: num(v.startedAt, 0, 1e14),
    endedAt: num(v.endedAt, 0, 1e14),
    timeSec: num(v.timeSec, 0, 1e8),
    profit: num(v.profit),
    maps: num(v.maps, 0, 1e7),
    lootCount: num(v.lootCount, 0, 1e9),
  };
}

export interface PersistedCurrentMap {
  levelType: number;
  levelId: number;
  startedAt: number;
  profit: number;
  timeSec: number;
  drops: RecordDrop[];
  expenses: RecordExpense[];
  timeline: TimelinePoint[];
}

export interface PersistedState {
  session: {
    id: string;
    startedAt: number;
    totalProfit: number;
    lootCount: number;
    mapCount: number;
    timeSec: number;
    inventoryValue: number;
    drops: RecordDrop[];
  };
  currentMap: PersistedCurrentMap | null;
  maps: MapRecord[];
  sessions: SessionRecord[];
  consumables: string[];
}

export function sanitizeState(raw: unknown): PersistedState | null {
  if (!isObj(raw) || raw.v !== 1 || !isObj(raw.session)) return null;
  const s = raw.session;
  const sessionId = safeId(s.id);
  if (!sessionId) return null;

  let currentMap: PersistedCurrentMap | null = null;
  if (isObj(raw.currentMap)) {
    const c = raw.currentMap;
    currentMap = {
      levelType: num(c.levelType, 0, 1000),
      levelId: num(c.levelId, 0, 1e12),
      startedAt: num(c.startedAt, 0, 1e14),
      profit: num(c.profit),
      timeSec: num(c.timeSec, 0, 1e7),
      drops: sanitizeDrops(c.drops, 5000),
      expenses: sanitizeExpenses(c.expenses, 500),
      timeline: sanitizeTimeline(c.timeline),
    };
  }

  const maps: MapRecord[] = [];
  for (const m of list(raw.maps, MAX_MAP_HISTORY)) {
    const rec = sanitizeMapRecord(m);
    if (rec) maps.push(rec);
  }
  const sessions: SessionRecord[] = [];
  for (const m of list(raw.sessions, MAX_ARCHIVED_SESSIONS)) {
    const rec = sanitizeSessionRecord(m);
    if (rec) sessions.push(rec);
  }

  return {
    session: {
      id: sessionId,
      startedAt: num(s.startedAt, 0, 1e14),
      totalProfit: num(s.totalProfit),
      lootCount: num(s.lootCount, 0, 1e9),
      mapCount: num(s.mapCount, 0, 1e7),
      timeSec: num(s.timeSec, 0, 1e8),
      inventoryValue: num(s.inventoryValue, 0),
      drops: sanitizeDrops(s.drops, 20000),
    },
    currentMap,
    maps,
    sessions,
    consumables: list(raw.consumables, 500).map(itemId).filter(Boolean),
  };
}
