// Item helpers: names, sprites, search.
import type { GameData, Item } from "./api";

const isWindows = navigator.userAgent.includes("Windows");
/** URL of the custom image protocol served by the Rust backend. */
export const asset = (path: string) => (isWindows ? `http://nhse.localhost/${path}` : `nhse://localhost/${path}`);

export const itemSprite = (id: number, variant = 0) => asset(`item/${id}?v=${variant}`);
export const villagerSprite = (internal: string) => asset(`villager/${internal}`);

export const NONE_ID = 0xfffe;
export const EXT_ID = 0xfffd;

export const count = (i: Item) => i.free & 0xffff;
export const withCount = (i: Item, c: number): Item => ({ ...i, free: (i.free & 0xffff0000) | (c & 0xffff) });
export const variantOf = (i: Item) => i.free & 7;
export const displayId = (i: Item) => (i.id === EXT_ID ? i.free & 0xffff : i.id);

export function baseName(gd: GameData, id: number): string {
  if (id === NONE_ID) return "(None)";
  if (id < gd.items.length) return gd.items[id];
  const f = gd.field_items.find((x) => x.id === id);
  if (f) return f.name;
  return `??? (0x${id.toString(16).toUpperCase()})`;
}

export function kindName(gd: GameData, id: number): string {
  if (id < gd.kinds.length) return gd.kind_names[gd.kinds[id]] ?? "";
  const f = gd.field_items.find((x) => x.id === id);
  return f ? f.kind_name : "";
}

export function itemColor(gd: GameData, id: number): string {
  if (id === NONE_ID) return "transparent";
  let c = 0x32cd32;
  if (id < gd.colors.length) c = gd.colors[id];
  else {
    const f = gd.field_items.find((x) => x.id === id);
    if (f) c = f.color;
  }
  return "#" + c.toString(16).padStart(6, "0");
}

/** Full descriptive name (recipe / fossil / tree details). */
export function itemLabel(gd: GameData, it: Item): string {
  if (it.id === NONE_ID) return "Empty";
  const id = displayId(it);
  const name = baseName(gd, id);
  if (id >= gd.kinds.length) return name;
  const k = gd.kinds[id];
  const sk = gd.special_kinds;
  if (k === sk.diy_recipe || k === sk.message_bottle) {
    const r = gd.recipes.find(([idx]) => idx === (it.free & 0xffff));
    return `${name} · ${r ? baseName(gd, r[1]) : "#" + (it.free & 0xffff)}`;
  }
  if (k === sk.fossil_unknown) return `${name} · ${baseName(gd, it.free & 0xffff)}`;
  if (k === sk.tree && count(it) !== 0) return `${name} · ${baseName(gd, count(it))}`;
  return name;
}

export function footprint(gd: GameData, it: Item): [number, number] {
  if (it.system & 0x24) return [2, 2];
  const id = displayId(it);
  let w = 2, h = 2;
  if (id < gd.kinds.length) { w = gd.sizes[id * 2]; h = gd.sizes[id * 2 + 1]; }
  if (it.system & 1) [w, h] = [h, w];
  return [w, h];
}

export const hex = (n: number, w = 4) => n.toString(16).toUpperCase().padStart(w, "0");

// ---------------------------------------------------------------- search

const strip = (s: string) => s.normalize("NFKD").replace(/[̀-ͯ]/g, "").toLowerCase();

export interface SearchEntry { id: number; name: string; key: string; }

export function buildIndex(gd: GameData, includeUnnamed = false, includeField = true): SearchEntry[] {
  const out: SearchEntry[] = [];
  for (let i = 0; i < gd.items.length; i++) {
    if (!includeUnnamed && !gd.named[i]) continue;
    out.push({ id: i, name: gd.items[i], key: strip(gd.items[i]) });
  }
  out.push({ id: NONE_ID, name: "(None)", key: "(none)" });
  if (includeField) for (const f of gd.field_items) out.push({ id: f.id, name: f.name, key: strip(f.name) });
  return out;
}

/**
 * Searches item names. Supports:
 *  - plain words (all must match, any order; accents ignored)
 *  - wildcards: `*` (any run) and `?` (one char), e.g. `* painting`, `?? statue`
 *  - regular expressions: `/^wooden.*chair$/`
 *  - ids: `0x0A54`, `#2626` or a bare number
 */
export function searchItems(index: SearchEntry[], query: string, limit = 400): SearchEntry[] {
  const q = query.trim();
  if (!q) return index.slice(0, limit);
  const idm = /^(?:0x|#)?([0-9a-f]{1,4})$/i.exec(q);
  const results: [number, SearchEntry][] = [];
  if (/^0x[0-9a-f]+$/i.test(q) || /^#\d+$/.test(q)) {
    const id = q.startsWith("#") ? parseInt(q.slice(1), 10) : parseInt(q, 16);
    return index.filter((e) => e.id === id);
  }
  let re: RegExp | null = null;
  if (q.length > 2 && q.startsWith("/") && q.lastIndexOf("/") > 0) {
    try { re = new RegExp(q.slice(1, q.lastIndexOf("/")), q.slice(q.lastIndexOf("/") + 1) || "i"); } catch { re = null; }
  } else if (/[*?]/.test(q)) {
    const pat = strip(q).split("").map((c) => (c === "*" ? ".*" : c === "?" ? "." : c.replace(/[.+^${}()|[\]\\]/g, "\\$&"))).join("");
    re = new RegExp("^" + pat + "$");
  }
  if (re) {
    for (const e of index) if (re.test(re.flags.includes("i") ? e.name : e.key)) results.push([0, e]);
    return results.slice(0, limit).map((r) => r[1]);
  }
  const words = strip(q).split(/\s+/).filter(Boolean);
  const full = strip(q);
  for (const e of index) {
    let ok = true;
    for (const w of words) if (!e.key.includes(w)) { ok = false; break; }
    if (!ok) continue;
    let score = 3;
    if (e.key === full) score = 0;
    else if (e.key.startsWith(full)) score = 1;
    else if (e.key.includes(" " + words[0]) || e.key.startsWith(words[0])) score = 2;
    results.push([score * 1000 + e.key.length, e]);
  }
  if (idm && results.length === 0) {
    const id = parseInt(idm[1], 16);
    return index.filter((e) => e.id === id);
  }
  results.sort((a, b) => a[0] - b[0]);
  return results.slice(0, limit).map((r) => r[1]);
}

export const fmtNum = (n: number) => n.toLocaleString();
export const fmtBytes = (n: number) => (n > 1e6 ? (n / 1e6).toFixed(1) + " MB" : n > 1e3 ? (n / 1e3).toFixed(0) + " KB" : n + " B");
export const fmtDate = (d: { year: number; month: number; day: number }) =>
  d.year === 0 && d.month === 0 && d.day === 0 ? "—" : `${d.year}-${String(d.month).padStart(2, "0")}-${String(d.day).padStart(2, "0")}`;
