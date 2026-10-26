// Column-major item grid helpers shared by the map and room editors (mirrors nhse-core's ItemGrid).
import type { GameData, Item } from "./api";
import { EXT_ID, NONE_ID, footprint } from "./items";

export const NONE_ITEM: Item = { id: NONE_ID, system: 0, additional: 0, free: 0 };
export const extensionOf = (rootId: number, x: number, y: number): Item => ({ id: EXT_ID, system: 0, additional: 0, free: ((rootId & 0xffff) | (x << 16) | (y << 24)) >>> 0 });

export function readItems(bytes: Uint8Array): Item[] {
  const dv = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const out: Item[] = new Array(bytes.length >> 3);
  for (let i = 0; i < out.length; i++) {
    const o = i * 8;
    out[i] = { id: dv.getUint16(o, true), system: dv.getUint8(o + 2), additional: dv.getUint8(o + 3), free: dv.getUint32(o + 4, true) };
  }
  return out;
}

export function writeItems(items: Item[]): Uint8Array {
  const out = new Uint8Array(items.length * 8);
  const dv = new DataView(out.buffer);
  items.forEach((it, i) => {
    const o = i * 8;
    dv.setUint16(o, it.id, true); dv.setUint8(o + 2, it.system); dv.setUint8(o + 3, it.additional); dv.setUint32(o + 4, it.free >>> 0, true);
  });
  return out;
}

export type PlaceResult = "ok" | "collision" | "bounds";

export class Grid {
  constructor(public w: number, public h: number, public tiles: Item[]) {}
  idx(x: number, y: number) { return x * this.h + y; }
  inside(x: number, y: number) { return x >= 0 && y >= 0 && x < this.w && y < this.h; }
  get(x: number, y: number): Item { return this.inside(x, y) ? this.tiles[this.idx(x, y)] : NONE_ITEM; }
  set(x: number, y: number, it: Item) { if (this.inside(x, y)) this.tiles[this.idx(x, y)] = it; }
  clone() { return new Grid(this.w, this.h, this.tiles.slice()); }

  /** Root tile of the item covering (x, y), following extension tiles. */
  rootOf(x: number, y: number): [number, number] | null {
    const t = this.get(x, y);
    if (t.id === NONE_ID) return null;
    if (t.id !== EXT_ID) return [x, y];
    const rx = x - ((t.free >>> 16) & 0xff), ry = y - ((t.free >>> 24) & 0xff);
    const r = this.get(rx, ry);
    return this.inside(rx, ry) && r.id === (t.free & 0xffff) ? [rx, ry] : null;
  }

  /** Checks whether `it` fits at (x, y); `ignore` is the root of an item being moved/replaced. */
  check(gd: GameData, it: Item, x: number, y: number, allowOverhang = false, ignore?: [number, number]): PlaceResult {
    const [w, h] = footprint(gd, it);
    if (!this.inside(x, y)) return "bounds";
    if (!allowOverhang && (x + w > this.w || y + h > this.h)) return "bounds";
    for (let ix = 0; ix < w; ix++) for (let iy = 0; iy < h; iy++) {
      if (!this.inside(x + ix, y + iy)) continue;
      const t = this.get(x + ix, y + iy);
      if (t.id === NONE_ID) continue;
      if (ignore) {
        const r = this.rootOf(x + ix, y + iy);
        if (r && r[0] === ignore[0] && r[1] === ignore[1]) continue;
      }
      return "collision";
    }
    return "ok";
  }

  /** Places an item; with `ext` its extension tiles are written too (clamped to the grid). */
  place(gd: GameData, it: Item, x: number, y: number, ext = true) {
    if (ext && it.id !== NONE_ID && it.id !== EXT_ID) {
      const [w, h] = footprint(gd, it);
      for (let ix = 0; ix < w; ix++) for (let iy = 0; iy < h; iy++) {
        if (ix === 0 && iy === 0) continue;
        this.set(x + ix, y + iy, extensionOf(it.id, ix, iy));
      }
    }
    this.set(x, y, it);
  }

  /** Removes the item covering (x, y) and its extensions. Returns the removed root item. */
  removeAt(gd: GameData, x: number, y: number): Item | null {
    const r = this.rootOf(x, y);
    if (!r) {
      const t = this.get(x, y);
      if (t.id === NONE_ID) return null;
      this.set(x, y, NONE_ITEM); // dangling extension
      return t;
    }
    const root = this.get(r[0], r[1]);
    const [w, h] = footprint(gd, root);
    for (let ix = 0; ix < w; ix++) for (let iy = 0; iy < h; iy++) {
      const t = this.get(r[0] + ix, r[1] + iy);
      if ((ix || iy) && t.id === EXT_ID && (t.free & 0xffff) === root.id) this.set(r[0] + ix, r[1] + iy, NONE_ITEM);
    }
    this.set(r[0], r[1], NONE_ITEM);
    return root;
  }

  extract(x0: number, y0: number, w: number, h: number): Item[] {
    const out: Item[] = [];
    for (let x = 0; x < w; x++) for (let y = 0; y < h; y++) out.push(this.get(x0 + x, y0 + y));
    return out;
  }

  paste(x0: number, y0: number, w: number, h: number, data: Item[], skipNone = true) {
    let n = 0;
    for (let x = 0; x < w; x++) for (let y = 0; y < h; y++) {
      const it = data[x * h + y];
      if (skipNone && it.id === NONE_ID) continue;
      if (!this.inside(x0 + x, y0 + y)) continue;
      this.set(x0 + x, y0 + y, it); n++;
    }
    return n;
  }
}
