import { Brush, Building2, Check as CheckIcon, Eye, ClipboardPaste, Copy, Crosshair, Download, Droplets, Eraser, Grid3x3, Hand, Layers, Map as MapIcon, MousePointer2, Redo2, Replace, RotateCw, Scissors, Sparkles, SquareDashed, Trash2, Undo2, Upload, ZoomIn, ZoomOut } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { PasteListModal } from "../components/ItemGridEditor";
import { ItemEditor } from "../components/ItemEditor";
import { ItemPicker } from "../components/ItemPicker";
import { Badge, Button, Callout, Check, Field, Loading, MenuButton, NumberInput, Segmented, Select, Tabs } from "../components/ui";
import { enumOptions, prettify } from "../components/Widgets";
import { api, type Arrangement, type Building, type ClearKind, type DumpKind, type Item, type MapMeta, type Rect } from "../lib/api";
import { F, pickFile, saveFile } from "../lib/dialog";
import { Grid, NONE_ITEM, readItems, writeItems } from "../lib/grid";
import { EXT_ID, NONE_ID, footprint, hex, itemColor, itemLabel, itemSprite, variantOf } from "../lib/items";
import { useGD, useStore } from "../lib/store";

type Tool = "inspect" | "place" | "erase" | "rect" | "pan";
type Panel = "tile" | "bulk" | "buildings" | "island" | "dumps";
type Pick = { kind: "building"; index: number } | { kind: "plaza" } | null;
interface View { s: number; x: number; y: number }

const MAP_W = 9 * 32, MAP_H = 8 * 32; // whole map in item tiles
const PLAZA_W = 12, PLAZA_H = 10;
const MIN_S = 1, MAX_S = 40;
const CLEAR_KINDS: [ClearKind, string][] = [
  ["weeds", "Weeds"], ["branches", "Tree branches"], ["stones", "Stones & rocks"], ["shells", "Shells"], ["flowers", "Flowers"], ["bushes", "Bushes"],
  ["trees", "Trees"], ["plants", "All plants"], ["fences", "Fences"], ["holes", "Holes"], ["field_objects", "Field objects"], ["placed_items", "Placed items"], ["everything", "Everything"],
];

const rgbCache = new Map<number, [number, number, number]>();

export default function MapEditor() {
  const gd = useGD();
  const { save, touch, toast, confirm, settings, rev } = useStore();
  const L = save!.field;
  const [meta, setMeta] = useState<MapMeta | null>(null);
  const [buildings, setBuildings] = useState<Building[] | null>(null);
  const [terrain, setTerrain] = useState<HTMLImageElement | null>(null);
  const layers = useRef<Item[][]>([[], []]);
  const [, setVer] = useState(0); // bumps whenever layer contents change (forces a redraw)
  const [layer, setLayer] = useState(0);
  const [tool, setTool] = useState<Tool>("inspect");
  const [panel, setPanel] = useState<Panel>("tile");
  const [view, setView] = useState<View>({ s: 3, x: 20, y: 20 });
  const [hover, setHover] = useState<[number, number] | null>(null); // layer coords
  const [selTile, setSelTile] = useState<[number, number] | null>(null);
  const [rect, setRect] = useState<Rect | null>(null);
  const [brush, setBrush] = useState<Item>({ ...NONE_ITEM });
  const [show, setShow] = useState({ other: true, grid: true, buildings: true, plaza: true, sprites: true });
  const [pick, setPick] = useState<Pick>(null);
  const [selBuilding, setSelBuilding] = useState<number | null>(null);
  const [floating, setFloating] = useState<[number, number][]>([]);
  const undo = useRef<{ layer: number; tiles: Item[] }[]>([]);
  const redo = useRef<{ layer: number; tiles: Item[] }[]>([]);
  const clip = useRef<{ w: number; h: number; tiles: Item[] } | null>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const wrap = useRef<HTMLDivElement>(null);
  const layerCanvas = useRef<(HTMLCanvasElement | null)[]>([null, null]);
  const sprites = useRef(new Map<string, HTMLImageElement | null>());
  const syncChain = useRef<Promise<unknown>>(Promise.resolve());
  const [loaded, setLoaded] = useState(false);
  // side panel state lives here so it survives the frequent re-renders caused by the canvas
  const [bs, setBs] = useState({
    kind: "weeds" as ClearKind, from: NONE_ID, to: NONE_ID, idOnly: true, src: "item" as "item" | "diy" | "list", spawnItem: { ...NONE_ITEM } as Item, count: 10,
    diy: { min: 0, max: 1000, alphabetical: true, count: 1 }, list: [] as Item[], dropped: true, arrange: "square" as Arrangement, overwrite: false, pasting: false,
  });
  const [showEmpty, setShowEmpty] = useState(false);
  const [acreSel, setAcreSel] = useState<[number, number]>([0, 0]);

  // ------------------------------------------------------------- loading
  const loadTerrain = useCallback(async () => {
    const png = await api.terrainPng(8);
    const url = URL.createObjectURL(new Blob([png as BlobPart], { type: "image/png" }));
    const img = new Image();
    img.onload = () => setTerrain(img);
    img.src = url;
  }, []);
  const loadLayer = useCallback(async (l: number) => { layers.current[l] = readItems(await api.fieldLayer(l)); layerCanvas.current[l] = null; }, []);
  useEffect(() => {
    (async () => {
      try {
        const [m, b] = await Promise.all([api.mapMeta(), api.buildings(), loadLayer(0), loadLayer(1), loadTerrain()]);
        setMeta(m); setBuildings(b); setVer((v) => v + 1); setLoaded(true);
        undo.current = []; redo.current = [];
      } catch (e) { toast("error", "Could not load the map", String(e)); }
    })();
  }, [rev]);

  // fit the map to the viewport on first load
  useEffect(() => {
    if (!loaded || !wrap.current) return;
    const r = wrap.current.getBoundingClientRect();
    const s = Math.max(MIN_S, Math.min((r.width - 40) / MAP_W, (r.height - 40) / MAP_H));
    setView({ s, x: (r.width - MAP_W * s) / 2, y: (r.height - MAP_H * s) / 2 });
  }, [loaded]);

  const grid = (l = layer) => new Grid(L.width, L.height, layers.current[l]);
  const rgb = (id: number): [number, number, number] => {
    let c = rgbCache.get(id);
    if (!c) { const h = itemColor(gd, id); const n = parseInt(h.slice(1), 16); c = [(n >> 16) & 255, (n >> 8) & 255, n & 255]; rgbCache.set(id, c); }
    return c;
  };
  const layerImage = (l: number) => {
    let c = layerCanvas.current[l];
    if (c) return c;
    c = document.createElement("canvas");
    c.width = L.width; c.height = L.height;
    const ctx = c.getContext("2d")!;
    const img = ctx.createImageData(L.width, L.height);
    const tiles = layers.current[l];
    for (let x = 0; x < L.width; x++) for (let y = 0; y < L.height; y++) {
      const it = tiles[x * L.height + y];
      if (!it || it.id === NONE_ID) continue;
      const [r, g, b] = rgb(it.id === EXT_ID ? it.free & 0xffff : it.id);
      const o = (y * L.width + x) * 4;
      img.data[o] = r; img.data[o + 1] = g; img.data[o + 2] = b; img.data[o + 3] = 235;
    }
    ctx.putImageData(img, 0, 0);
    layerCanvas.current[l] = c;
    return c;
  };
  const changed = (l: number) => { layerCanvas.current[l] = null; setVer((v) => v + 1); };
  const sync = (l: number) => {
    const bytes = writeItems(layers.current[l]);
    syncChain.current = syncChain.current.then(() => api.setFieldLayer(l, bytes)).then(touch).catch((e) => toast("error", "Could not save map changes", String(e)));
    return syncChain.current;
  };
  const snapshot = (l = layer) => { undo.current.push({ layer: l, tiles: layers.current[l].slice() }); if (undo.current.length > 100) undo.current.shift(); redo.current = []; };
  const doUndo = (from: typeof undo, to: typeof undo) => {
    const s = from.current.pop();
    if (!s) return;
    to.current.push({ layer: s.layer, tiles: layers.current[s.layer].slice() });
    layers.current[s.layer] = s.tiles;
    changed(s.layer); sync(s.layer);
  };
  /** Runs a backend bulk operation and reloads the layer. */
  const bulk = async (label: string, l: number, op: () => Promise<number>) => {
    await syncChain.current;
    snapshot(l);
    try {
      const n = await op();
      await loadLayer(l); changed(l); touch();
      toast(n ? "ok" : "info", n ? `${label}: ${n} tile${n === 1 ? "" : "s"} changed` : `${label}: nothing to change`);
    } catch (e) { undo.current.pop(); toast("error", label + " failed", String(e)); }
  };

  // ------------------------------------------------------------- coordinates
  const toTile = (cx: number, cy: number): [number, number] => {
    const mx = Math.floor((cx - view.x) / view.s), my = Math.floor((cy - view.y) / view.s);
    return [mx - L.offset_x, my - L.offset_y];
  };
  const inLayer = (x: number, y: number) => x >= 0 && y >= 0 && x < L.width && y < L.height;

  // ------------------------------------------------------------- drawing
  useEffect(() => {
    const c = canvas.current, w = wrap.current;
    if (!c || !w || !loaded) return;
    let frame = 0;
    const draw = () => {
      const dpr = window.devicePixelRatio || 1;
      const { width: cw, height: ch } = w.getBoundingClientRect();
      if (c.width !== Math.round(cw * dpr) || c.height !== Math.round(ch * dpr)) { c.width = Math.round(cw * dpr); c.height = Math.round(ch * dpr); c.style.width = cw + "px"; c.style.height = ch + "px"; }
      const ctx = c.getContext("2d")!;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.imageSmoothingEnabled = false;
      ctx.fillStyle = getComputedStyle(document.body).getPropertyValue("--bg") || "#111";
      ctx.fillRect(0, 0, cw, ch);
      const { s, x: px, y: py } = view;
      const ox = px + L.offset_x * s, oy = py + L.offset_y * s;
      if (terrain) ctx.drawImage(terrain, 0, 0, terrain.width, terrain.height, px, py, MAP_W * s, MAP_H * s);
      if (show.plaza && meta) {
        ctx.fillStyle = "rgba(188,143,143,0.55)";
        ctx.fillRect(px + meta.plaza.x * s, py + meta.plaza.z * s, PLAZA_W * s, PLAZA_H * s);
      }
      if (show.other) { ctx.globalAlpha = 0.3; ctx.drawImage(layerImage(1 - layer), ox, oy, L.width * s, L.height * s); }
      ctx.globalAlpha = 0.92;
      ctx.drawImage(layerImage(layer), ox, oy, L.width * s, L.height * s);
      ctx.globalAlpha = 1;

      // visible range in layer coords
      const x0 = Math.max(0, Math.floor(-ox / s)), y0 = Math.max(0, Math.floor(-oy / s));
      const x1 = Math.min(L.width, Math.ceil((cw - ox) / s)), y1 = Math.min(L.height, Math.ceil((ch - oy) / s));
      const tiles = layers.current[layer];
      if (show.sprites && s >= 12 && (x1 - x0) * (y1 - y0) < 12000) {
        for (let x = Math.max(0, x0 - 4); x < x1; x++) for (let y = Math.max(0, y0 - 4); y < y1; y++) {
          const it = tiles[x * L.height + y];
          if (!it || it.id === NONE_ID || it.id === EXT_ID) continue;
          const [fw, fh] = footprint(gd, it);
          const key = `${it.id}:${variantOf(it)}`;
          let img = sprites.current.get(key);
          if (img === undefined) {
            const im = new Image();
            sprites.current.set(key, null);
            im.onload = () => { sprites.current.set(key, im); setVer((v) => v + 1); };
            im.src = itemSprite(it.id, variantOf(it));
            img = null;
          }
          ctx.strokeStyle = "rgba(0,0,0,0.35)"; ctx.lineWidth = 1;
          ctx.strokeRect(ox + x * s + 0.5, oy + y * s + 0.5, fw * s - 1, fh * s - 1);
          if (img) {
            const d = Math.min(fw, fh) * s * 0.9;
            ctx.imageSmoothingEnabled = true;
            ctx.drawImage(img, ox + x * s + (fw * s - d) / 2, oy + y * s + (fh * s - d) / 2, d, d);
            ctx.imageSmoothingEnabled = false;
          }
        }
      }
      // grid lines
      if (show.grid) {
        ctx.lineWidth = 1;
        if (s >= 8) {
          ctx.strokeStyle = "rgba(255,255,255,0.10)";
          ctx.beginPath();
          for (let x = x0; x <= x1; x++) { ctx.moveTo(Math.round(ox + x * s) + 0.5, oy + y0 * s); ctx.lineTo(Math.round(ox + x * s) + 0.5, oy + y1 * s); }
          for (let y = y0; y <= y1; y++) { ctx.moveTo(ox + x0 * s, Math.round(oy + y * s) + 0.5); ctx.lineTo(ox + x1 * s, Math.round(oy + y * s) + 0.5); }
          ctx.stroke();
        }
        ctx.strokeStyle = "rgba(255,255,255,0.45)";
        ctx.setLineDash([4, 4]);
        ctx.beginPath();
        for (let a = 0; a <= 9; a++) { const X = Math.round(px + a * 32 * s) + 0.5; ctx.moveTo(X, py); ctx.lineTo(X, py + MAP_H * s); }
        for (let a = 0; a <= 8; a++) { const Y = Math.round(py + a * 32 * s) + 0.5; ctx.moveTo(px, Y); ctx.lineTo(px + MAP_W * s, Y); }
        ctx.stroke();
        ctx.setLineDash([]);
        if (s >= 2.5) {
          ctx.fillStyle = "rgba(255,255,255,0.75)";
          ctx.font = `600 ${Math.min(14, 4 + s * 1.5)}px Inter, system-ui`;
          for (let ay = 0; ay < 8; ay++) for (let ax = 0; ax < 9; ax++) ctx.fillText(String.fromCharCode(65 + ax) + (ay + 1), px + ax * 32 * s + 4, py + ay * 32 * s + Math.min(14, 4 + s * 1.5) + 2);
        }
        ctx.strokeStyle = "rgba(255,255,255,0.9)"; ctx.lineWidth = 1.5;
        ctx.strokeRect(ox, oy, L.width * s, L.height * s);
      }
      // floating items
      if (floating.length && layer === 1) {
        ctx.strokeStyle = "#ff4d6d"; ctx.lineWidth = 2;
        for (const [x, y] of floating) ctx.strokeRect(ox + x * s, oy + y * s, s, s);
      }
      // buildings
      if (show.buildings && buildings) {
        ctx.font = `600 ${Math.max(10, Math.min(13, s * 2))}px Inter, system-ui`;
        ctx.textAlign = "center";
        buildings.forEach((b, i) => {
          if (!b.kind) return;
          const cx = px + b.x * s, cy = py + b.y * s, r = Math.max(5, 3 * s);
          ctx.beginPath(); ctx.arc(cx, cy, r, 0, Math.PI * 2);
          ctx.fillStyle = i === selBuilding ? "rgba(255,170,0,0.85)" : "rgba(30,30,40,0.55)";
          ctx.fill();
          ctx.strokeStyle = i === selBuilding ? "#fff" : "rgba(255,255,255,0.8)"; ctx.lineWidth = 1.5; ctx.stroke();
          if (s >= 2) {
            const label = buildingName(b.kind);
            ctx.fillStyle = "rgba(0,0,0,0.55)";
            const tw = ctx.measureText(label).width + 8;
            ctx.fillRect(cx - tw / 2, cy + r + 2, tw, 16);
            ctx.fillStyle = "#fff"; ctx.fillText(label, cx, cy + r + 14);
          }
        });
        ctx.textAlign = "start";
      }
      // selection rectangle
      if (rect) {
        ctx.fillStyle = "rgba(80,160,255,0.15)"; ctx.strokeStyle = "#4da3ff"; ctx.lineWidth = 1.5; ctx.setLineDash([6, 3]);
        ctx.fillRect(ox + rect.x * s, oy + rect.y * s, rect.w * s, rect.h * s);
        ctx.strokeRect(ox + rect.x * s + 0.5, oy + rect.y * s + 0.5, rect.w * s, rect.h * s);
        ctx.setLineDash([]);
      }
      // selected tile
      if (selTile) {
        const it = tiles[selTile[0] * L.height + selTile[1]];
        const [fw, fh] = it && it.id !== NONE_ID ? footprint(gd, it) : [1, 1];
        ctx.strokeStyle = "#ffd23f"; ctx.lineWidth = 2;
        ctx.strokeRect(ox + selTile[0] * s - 1, oy + selTile[1] * s - 1, fw * s + 2, fh * s + 2);
      }
      // hover
      if (hover && inLayer(hover[0], hover[1])) {
        let [fw, fh] = [1, 1];
        let ok = true;
        if (tool === "place" && brush.id !== NONE_ID) {
          [fw, fh] = footprint(gd, brush);
          ok = !settings?.map_prevent_overlap || grid().check(gd, brush, hover[0], hover[1]) === "ok";
          if (hover[0] + fw > L.width || hover[1] + fh > L.height) ok = false;
        } else if (tool === "rect" && clip.current && pasteMode.current) {
          [fw, fh] = [clip.current.w, clip.current.h];
        }
        ctx.strokeStyle = ok ? "rgba(255,255,255,0.95)" : "#ff4d6d"; ctx.lineWidth = 1.5;
        ctx.fillStyle = ok ? "rgba(255,255,255,0.18)" : "rgba(255,77,109,0.25)";
        ctx.fillRect(ox + hover[0] * s, oy + hover[1] * s, fw * s, fh * s);
        ctx.strokeRect(ox + hover[0] * s + 0.5, oy + hover[1] * s + 0.5, fw * s - 1, fh * s - 1);
      }
    };
    frame = requestAnimationFrame(draw);
    const ro = new ResizeObserver(() => { cancelAnimationFrame(frame); frame = requestAnimationFrame(draw); });
    ro.observe(w);
    return () => { cancelAnimationFrame(frame); ro.disconnect(); };
  });

  const buildingName = (k: number) => prettify((gd.enums.BuildingType ?? []).find(([, v]) => v === k)?.[0] ?? `Type ${k}`);

  // ------------------------------------------------------------- interaction
  const drag = useRef<{ mode: "pan" | "paint" | "erase" | "rect"; sx: number; sy: number; vx: number; vy: number; tx: number; ty: number; dirty: boolean; last: string } | null>(null);
  const pasteMode = useRef(false);
  const space = useRef(false);

  const paintAt = (x: number, y: number, single: boolean) => {
    const g = grid();
    if (brush.id === NONE_ID) { if (single) toast("info", "Choose an item to place first", "Pick it in the side panel, or select a tile and use “Use as brush”."); return false; }
    const [fw, fh] = footprint(gd, brush);
    if (!inLayer(x, y) || x + fw > L.width || y + fh > L.height) return false;
    if (settings?.map_prevent_overlap) {
      if (g.check(gd, brush, x, y) !== "ok") { if (single) toast("warn", "That spot is occupied", "Turn off “prevent overlapping items” in Settings to overwrite."); return false; }
    } else {
      for (let ix = 0; ix < fw; ix++) for (let iy = 0; iy < fh; iy++) g.removeAt(gd, x + ix, y + iy);
    }
    g.place(gd, { ...brush }, x, y, settings?.map_auto_extensions ?? true);
    return true;
  };
  const eraseAt = (x: number, y: number) => inLayer(x, y) && !!grid().removeAt(gd, x, y);

  const onDown = (e: React.PointerEvent) => {
    const r = canvas.current!.getBoundingClientRect();
    const cx = e.clientX - r.left, cy = e.clientY - r.top;
    const [tx, ty] = toTile(cx, cy);
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
    if (e.button === 1 || tool === "pan" || space.current) { drag.current = { mode: "pan", sx: cx, sy: cy, vx: view.x, vy: view.y, tx, ty, dirty: false, last: "" }; return; }
    if (e.button !== 0) return;
    if (pick) {
      const mx = tx + L.offset_x, my = ty + L.offset_y;
      if (pick.kind === "plaza" && meta) { const plaza = { x: Math.max(0, mx), z: Math.max(0, my) }; setMeta({ ...meta, plaza }); api.setPlaza(plaza).then(touch); }
      if (pick.kind === "building" && buildings) { const next = buildings.slice(); next[pick.index] = { ...next[pick.index], x: Math.max(0, mx), y: Math.max(0, my) }; setBuildings(next); api.setBuildings(next).then(touch); }
      setPick(null);
      return;
    }
    if (tool === "inspect") {
      if (show.buildings && buildings) {
        const hit = buildings.findIndex((b) => b.kind && Math.hypot(view.x + b.x * view.s - cx, view.y + b.y * view.s - cy) <= Math.max(5, 3 * view.s));
        if (hit >= 0) { setSelBuilding(hit); setPanel("buildings"); return; }
      }
      if (inLayer(tx, ty)) { setSelTile(grid().rootOf(tx, ty) ?? [tx, ty]); setPanel("tile"); }
      return;
    }
    if (tool === "rect") {
      if (pasteMode.current && clip.current) { pasteClip(tx, ty); return; }
      drag.current = { mode: "rect", sx: cx, sy: cy, vx: 0, vy: 0, tx, ty, dirty: false, last: "" };
      setRect(inLayer(tx, ty) ? { x: tx, y: ty, w: 1, h: 1 } : null);
      return;
    }
    snapshot();
    const d = { mode: tool === "place" ? "paint" as const : "erase" as const, sx: cx, sy: cy, vx: 0, vy: 0, tx, ty, dirty: false, last: `${tx},${ty}` };
    d.dirty = tool === "place" ? paintAt(tx, ty, true) : eraseAt(tx, ty);
    drag.current = d;
    if (d.dirty) changed(layer);
  };

  const onMove = (e: React.PointerEvent) => {
    const r = canvas.current!.getBoundingClientRect();
    const cx = e.clientX - r.left, cy = e.clientY - r.top;
    const [tx, ty] = toTile(cx, cy);
    if (!hover || hover[0] !== tx || hover[1] !== ty) setHover([tx, ty]);
    const d = drag.current;
    if (!d) return;
    if (d.mode === "pan") { setView({ ...view, x: d.vx + cx - d.sx, y: d.vy + cy - d.sy }); return; }
    if (d.mode === "rect") {
      const x0 = Math.max(0, Math.min(d.tx, tx)), y0 = Math.max(0, Math.min(d.ty, ty));
      const x1 = Math.min(L.width - 1, Math.max(d.tx, tx)), y1 = Math.min(L.height - 1, Math.max(d.ty, ty));
      if (x1 >= x0 && y1 >= y0) setRect({ x: x0, y: y0, w: x1 - x0 + 1, h: y1 - y0 + 1 });
      return;
    }
    const key = `${tx},${ty}`;
    if (key === d.last) return;
    d.last = key;
    if (d.mode === "paint") {
      const [fw, fh] = footprint(gd, brush);
      if ((((tx - d.tx) % fw) + fw) % fw !== 0 || (((ty - d.ty) % fh) + fh) % fh !== 0) return; // stamp on the brush grid
      if (paintAt(tx, ty, false)) { d.dirty = true; changed(layer); }
    } else if (eraseAt(tx, ty)) { d.dirty = true; changed(layer); }
  };

  const onUp = () => {
    const d = drag.current;
    drag.current = null;
    if (!d) return;
    if (d.mode === "paint" || d.mode === "erase") {
      if (d.dirty) sync(layer); else undo.current.pop();
    }
  };

  // wheel: pinch / ⌘-scroll zooms, plain scroll pans (trackpad friendly)
  useEffect(() => {
    const c = canvas.current;
    if (!c) return;
    const wheel = (e: WheelEvent) => {
      e.preventDefault();
      const r = c.getBoundingClientRect();
      const cx = e.clientX - r.left, cy = e.clientY - r.top;
      setView((v) => {
        if (e.ctrlKey || e.metaKey) {
          const s = Math.min(MAX_S, Math.max(MIN_S, v.s * Math.exp(-e.deltaY * (e.ctrlKey ? 0.012 : 0.004))));
          return { s, x: cx - ((cx - v.x) / v.s) * s, y: cy - ((cy - v.y) / v.s) * s };
        }
        return { ...v, x: v.x - e.deltaX, y: v.y - e.deltaY };
      });
    };
    c.addEventListener("wheel", wheel, { passive: false });
    return () => c.removeEventListener("wheel", wheel);
  }, [loaded]);

  const zoomBy = (f: number) => {
    const r = wrap.current!.getBoundingClientRect();
    const cx = r.width / 2, cy = r.height / 2;
    setView((v) => { const s = Math.min(MAX_S, Math.max(MIN_S, v.s * f)); return { s, x: cx - ((cx - v.x) / v.s) * s, y: cy - ((cy - v.y) / v.s) * s }; });
  };

  const copyRect = (cut = false) => {
    if (!rect) return;
    clip.current = { w: rect.w, h: rect.h, tiles: grid().extract(rect.x, rect.y, rect.w, rect.h) };
    if (cut) { snapshot(); const g = grid(); for (let x = 0; x < rect.w; x++) for (let y = 0; y < rect.h; y++) g.set(rect.x + x, rect.y + y, NONE_ITEM); changed(layer); sync(layer); }
    toast("info", `${cut ? "Cut" : "Copied"} ${rect.w}×${rect.h} tiles`, "Press ⌘V, then click where the top-left corner should go.");
  };
  const pasteClip = (x: number, y: number) => {
    const c = clip.current;
    pasteMode.current = false;
    if (!c || !inLayer(x, y)) return;
    snapshot();
    const n = grid().paste(x, y, c.w, c.h, c.tiles, true);
    setRect({ x, y, w: Math.min(c.w, L.width - x), h: Math.min(c.h, L.height - y) });
    changed(layer); sync(layer);
    toast("ok", `Pasted ${n} tiles`);
  };
  const deleteRect = () => {
    if (!rect) return;
    snapshot();
    const g = grid();
    for (let x = rect.x; x < rect.x + rect.w; x++) for (let y = rect.y; y < rect.y + rect.h; y++) g.removeAt(gd, x, y);
    changed(layer); sync(layer);
  };

  // keyboard shortcuts
  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      const t = e.target as HTMLElement;
      if (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.tagName === "SELECT" || t.isContentEditable) return;
      const mod = e.metaKey || e.ctrlKey;
      if (e.code === "Space") { space.current = true; e.preventDefault(); return; }
      if (mod && e.key.toLowerCase() === "z") { e.preventDefault(); e.shiftKey ? doUndo(redo, undo) : doUndo(undo, redo); return; }
      if (mod && e.key.toLowerCase() === "y") { e.preventDefault(); doUndo(redo, undo); return; }
      if (mod && e.key.toLowerCase() === "c") { if (rect) { e.preventDefault(); copyRect(); } return; }
      if (mod && e.key.toLowerCase() === "x") { if (rect) { e.preventDefault(); copyRect(true); } return; }
      if (mod && e.key.toLowerCase() === "v") { if (clip.current) { e.preventDefault(); setTool("rect"); pasteMode.current = true; toast("info", "Click on the map to paste"); } return; }
      if (mod) return;
      const k = e.key.toLowerCase();
      if (k === "v") setTool("inspect"); else if (k === "b") setTool("place"); else if (k === "e") setTool("erase"); else if (k === "m") setTool("rect"); else if (k === "h") setTool("pan");
      else if (k === "r" && tool === "place") setBrush((b) => ({ ...b, system: (b.system & ~3) | (((b.system & 3) + 1) & 3) }));
      else if (k === "escape") { setRect(null); setPick(null); pasteMode.current = false; }
      else if ((k === "delete" || k === "backspace") && rect) deleteRect();
      else if (k === "1") setLayer(0); else if (k === "2") setLayer(1);
      else if (k === "=" || k === "+") zoomBy(1.25); else if (k === "-") zoomBy(0.8);
    };
    const up = (e: KeyboardEvent) => { if (e.code === "Space") space.current = false; };
    window.addEventListener("keydown", down); window.addEventListener("keyup", up);
    return () => { window.removeEventListener("keydown", down); window.removeEventListener("keyup", up); };
  });

  if (!loaded || !meta || !buildings) return <Loading />;
  const hovered = hover && inLayer(hover[0], hover[1]) ? grid().get(hover[0], hover[1]) : null;
  const acreLabel = (x: number, y: number) => { const mx = x + L.offset_x, my = y + L.offset_y; return String.fromCharCode(65 + Math.floor(mx / 32)) + (Math.floor(my / 32) + 1); };
  const cursor = pick ? "crosshair" : tool === "pan" ? "grab" : tool === "inspect" ? "default" : "crosshair";

  return (
    <div className="content flush" style={{ flexDirection: "row" }}>
      <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column" }}>
        <div className="row wrap" style={{ padding: "10px 14px", borderBottom: "1px solid var(--border)", background: "var(--card)", gap: 10 }}>
          <Segmented value={tool} onChange={(t) => { setTool(t); pasteMode.current = false; }} options={[
            { value: "inspect", label: "Select", icon: <MousePointer2 /> }, { value: "place", label: "Place", icon: <Brush /> },
            { value: "erase", label: "Erase", icon: <Eraser /> }, { value: "rect", label: "Area", icon: <SquareDashed /> }, { value: "pan", label: "Pan", icon: <Hand /> }]} />
          <Segmented value={layer} onChange={(l) => { setLayer(l); setSelTile(null); }} options={[{ value: 0, label: "Layer 1", icon: <Layers /> }, { value: 1, label: "Layer 2" }]} />
          <div className="row" style={{ gap: 4 }}>
            <Button size="sm" iconOnly icon={<Undo2 />} title="Undo (⌘Z)" onClick={() => doUndo(undo, redo)} disabled={!undo.current.length} />
            <Button size="sm" iconOnly icon={<Redo2 />} title="Redo (⇧⌘Z)" onClick={() => doUndo(redo, undo)} disabled={!redo.current.length} />
          </div>
          {tool === "rect" && rect && (
            <div className="row" style={{ gap: 4 }}>
              <Button size="sm" iconOnly icon={<Copy />} title="Copy (⌘C)" onClick={() => copyRect()} />
              <Button size="sm" iconOnly icon={<Scissors />} title="Cut (⌘X)" onClick={() => copyRect(true)} />
              <Button size="sm" iconOnly icon={<Trash2 />} title="Delete (⌫)" onClick={deleteRect} />
            </div>
          )}
          {tool === "rect" && clip.current && <Button size="sm" icon={<ClipboardPaste />} onClick={() => { pasteMode.current = true; toast("info", "Click on the map to paste"); }}>Paste</Button>}
          <div style={{ flex: 1 }} />
          <MenuButton label="View" icon={<Eye />} size="sm" align="right" items={([
            ["other", "Other layer (dimmed)"], ["grid", "Grid & acre names"], ["buildings", "Buildings"], ["plaza", "Plaza"], ["sprites", "Item pictures when zoomed in"],
          ] as [keyof typeof show, string][]).map(([k, label]) => ({ label, icon: show[k] ? <CheckIcon /> : <span style={{ width: 14 }} />, onClick: () => setShow({ ...show, [k]: !show[k] }) }))} />
          <div className="row" style={{ gap: 4 }}>
            <Button size="sm" iconOnly icon={<ZoomOut />} onClick={() => zoomBy(0.8)} title="Zoom out (−)" />
            <span className="small faint mono" style={{ width: 44, textAlign: "center" }}>{Math.round(view.s * 100 / 4)}%</span>
            <Button size="sm" iconOnly icon={<ZoomIn />} onClick={() => zoomBy(1.25)} title="Zoom in (+)" />
          </div>
        </div>
        <div ref={wrap} style={{ flex: 1, position: "relative", overflow: "hidden" }}>
          <canvas ref={canvas} style={{ position: "absolute", inset: 0, cursor, touchAction: "none" }}
            onPointerDown={onDown} onPointerMove={onMove} onPointerUp={onUp} onPointerCancel={onUp} onPointerLeave={() => setHover(null)} onContextMenu={(e) => e.preventDefault()} />
          {pick && <div style={{ position: "absolute", top: 12, left: "50%", transform: "translateX(-50%)" }}><Badge kind="accent">Click on the map to place the {pick.kind === "plaza" ? "plaza" : "building"} · Esc to cancel</Badge></div>}
          <div className="small" style={{ position: "absolute", left: 12, bottom: 10, padding: "4px 10px", borderRadius: 8, background: "color-mix(in srgb, var(--card) 88%, transparent)", border: "1px solid var(--border)", pointerEvents: "none" }}>
            {hover && inLayer(hover[0], hover[1]) ? <>Acre <b>{acreLabel(hover[0], hover[1])}</b> · X {hover[0]} Y {hover[1]} · {itemLabel(gd, hovered!)}{hovered && hovered.id !== NONE_ID && hovered.id !== EXT_ID && <span className="faint"> · 0x{hex(hovered.id)}</span>}</> : <span className="faint">Scroll to pan · pinch or ⌘-scroll to zoom · Space-drag to pan</span>}
          </div>
        </div>
      </div>

      <aside style={{ width: 360, flex: "none", borderLeft: "1px solid var(--border)", background: "var(--card)", display: "flex", flexDirection: "column", minHeight: 0 }}>
        <div style={{ padding: "6px 6px 0" }}>
          <Tabs value={panel} onChange={setPanel} tabs={[
            { value: "tile", label: "Item", icon: <Crosshair /> }, { value: "bulk", label: "Bulk", icon: <Sparkles /> },
            { value: "buildings", label: "Buildings", icon: <Building2 /> }, { value: "island", label: "Island", icon: <MapIcon /> }, { value: "dumps", label: "Files", icon: <Download /> }]} />
        </div>
        <div style={{ flex: 1, overflowY: "auto", padding: 16 }} className="col">
          {panel === "tile" && renderTile()}
          {panel === "bulk" && renderBulk()}
          {panel === "buildings" && renderBuildings()}
          {panel === "island" && renderIsland()}
          {panel === "dumps" && renderDumps()}
        </div>
      </aside>
    </div>
  );

  // ------------------------------------------------------------- panels (closures over editor state)
  function renderTile() {
    const selItem = selTile ? grid().get(selTile[0], selTile[1]) : null;
    return (
      <div className="col" style={{ gap: 16 }}>
        <div className="col" style={{ gap: 8 }}>
          <div className="row"><div className="label" style={{ flex: 1 }}>Brush</div><span className="tiny faint"><span className="kbd">B</span> place · <span className="kbd">R</span> rotate</span></div>
          <ItemEditor item={brush} onChange={setBrush} />
          <div className="row">
            <Button size="sm" icon={<RotateCw />} onClick={() => setBrush({ ...brush, system: (brush.system & ~3) | (((brush.system & 3) + 1) & 3) })}>Rotate</Button>
            <Button size="sm" icon={<Brush />} variant={tool === "place" ? "primary" : "default"} onClick={() => setTool("place")}>Paint with it</Button>
          </div>
        </div>
        <div className="divider" />
        {selTile && selItem ? (
          <div className="col" style={{ gap: 8 }}>
            <div className="label">Selected tile · {acreLabel(selTile[0], selTile[1])} · X {selTile[0]} Y {selTile[1]}</div>
            {selItem.id === NONE_ID ? <div className="faint small">Nothing here.</div> : selItem.id === EXT_ID ? (
              <Callout kind="warn" title="Orphaned extension tile">This tile belongs to a multi-tile item that no longer exists. <Button size="sm" onClick={() => { snapshot(); grid().set(selTile[0], selTile[1], NONE_ITEM); changed(layer); sync(layer); }}>Remove it</Button></Callout>
            ) : (
              <>
                <ItemEditor item={selItem} onChange={(it) => {
                  const g = grid().clone();
                  g.removeAt(gd, selTile[0], selTile[1]);
                  if (it.id !== NONE_ID) {
                    const fit = g.check(gd, it, selTile[0], selTile[1]);
                    if (fit === "bounds" || (fit === "collision" && settings?.map_prevent_overlap)) { toast("warn", "The changed item doesn't fit here"); return; }
                    g.place(gd, it, selTile[0], selTile[1], settings?.map_auto_extensions ?? true);
                  }
                  snapshot();
                  layers.current[layer] = g.tiles;
                  changed(layer); sync(layer);
                }} />
                <div className="row">
                  <Button size="sm" icon={<Brush />} onClick={() => { setBrush(selItem); setTool("place"); }}>Use as brush</Button>
                  <Button size="sm" variant="danger" icon={<Trash2 />} onClick={() => { snapshot(); grid().removeAt(gd, selTile[0], selTile[1]); changed(layer); sync(layer); setSelTile(null); }}>Remove</Button>
                </div>
              </>
            )}
          </div>
        ) : <div className="faint small">Use <b>Select</b> (<span className="kbd">V</span>) and click a tile to inspect and edit the item on it.</div>}
        <div className="divider" />
        <div className="tiny faint" style={{ lineHeight: 1.7 }}>
          <span className="kbd">V</span> select · <span className="kbd">B</span> place · <span className="kbd">E</span> erase · <span className="kbd">M</span> area · <span className="kbd">H</span> pan · <span className="kbd">1</span>/<span className="kbd">2</span> layer · <span className="kbd">⌘Z</span> undo · <span className="kbd">⌘C</span>/<span className="kbd">⌘V</span> copy/paste area
        </div>
      </div>
    );
  }

  function renderBulk() {
    const { kind, from, to, idOnly, src, spawnItem, count, diy, list, dropped, arrange, overwrite, pasting } = bs;
    const up = <K extends keyof typeof bs>(k: K) => (v: (typeof bs)[K]) => setBs((p) => ({ ...p, [k]: v }));
    const [setKind, setFrom, setTo, setIdOnly, setSrc, setSpawnItem, setCount, setDiy, setList, setDropped, setArrange, setOverwrite, setPasting] =
      [up("kind"), up("from"), up("to"), up("idOnly"), up("src"), up("spawnItem"), up("count"), up("diy"), up("list"), up("dropped"), up("arrange"), up("overwrite"), up("pasting")];
    const target = rect ?? undefined;
    const where = rect ? `selected area (${rect.w}×${rect.h})` : "whole layer";
    const origin = rect ? [rect.x, rect.y] : selTile;
    return (
      <div className="col" style={{ gap: 18 }}>
        <Callout kind="info">Applies to the <b>{where}</b> of layer {layer + 1}. Use the <b>Area</b> tool (<span className="kbd">M</span>) to limit it. Every action can be undone.</Callout>
        <section className="col">
          <div className="label">Remove</div>
          <div className="row">
            <Select value={kind} onChange={setKind} options={CLEAR_KINDS.map(([v, label]) => ({ value: v, label }))} style={{ flex: 1 }} />
            <Button icon={<Trash2 />} onClick={async () => {
              if (kind === "everything" && !(await confirm({ title: `Remove everything in the ${where}?`, danger: true, confirm: "Remove" }))) return;
              bulk("Remove " + CLEAR_KINDS.find(([k]) => k === kind)![1].toLowerCase(), layer, () => api.fieldClear(layer, kind, target));
            }}>Remove</Button>
          </div>
        </section>
        <section className="col">
          <div className="label">Flowers</div>
          <div className="row">
            <Button icon={<Droplets />} onClick={() => bulk("Water flowers", layer, () => api.fieldWater(layer, false, target))}>Water</Button>
            <Button icon={<Sparkles />} onClick={() => bulk("Gold-water flowers", layer, () => api.fieldWater(layer, true, target))} title="Watered by every visitor (best breeding odds)">Water by all visitors</Button>
          </div>
        </section>
        <section className="col">
          <div className="label">Replace</div>
          <Field label="Find"><ItemPicker value={from} onChange={setFrom} /></Field>
          <Field label="Replace with"><ItemPicker value={to} onChange={setTo} /></Field>
          <Check label="Match by item only (ignore variant, count, flags)" checked={idOnly} onChange={setIdOnly} />
          <Button icon={<Replace />} disabled={from === NONE_ID || to === NONE_ID} onClick={() => bulk("Replace", layer, () => api.fieldReplace(layer, { ...NONE_ITEM, id: from }, { ...NONE_ITEM, id: to }, idOnly, target))}>Replace all</Button>
        </section>
        <section className="col">
          <div className="label">Spawn items</div>
          <Segmented value={src} onChange={setSrc} options={[{ value: "item", label: "One item" }, { value: "diy", label: "DIY cards" }, { value: "list", label: "From a list" }]} />
          {src === "item" && <><ItemEditor item={spawnItem} compact onChange={setSpawnItem} /><Field label="How many"><NumberInput value={count} min={1} max={10000} onChange={setCount} /></Field></>}
          {src === "diy" && (
            <div className="grid cols-2" style={{ gap: 10 }}>
              <Field label="From recipe #"><NumberInput value={diy.min} max={0xffff} onChange={(min) => setDiy({ ...diy, min })} /></Field>
              <Field label="To recipe #"><NumberInput value={diy.max} max={0xffff} onChange={(max) => setDiy({ ...diy, max })} /></Field>
              <Field label="Copies of each"><NumberInput value={diy.count} min={1} max={20} onChange={(c) => setDiy({ ...diy, count: c })} /></Field>
              <Check label="Sort alphabetically" checked={diy.alphabetical} onChange={(alphabetical) => setDiy({ ...diy, alphabetical })} />
            </div>
          )}
          {src === "list" && (
            <div className="row">
              <Button size="sm" onClick={() => setPasting(true)}>Paste a list…</Button>
              <span className="small faint">{list.length} item{list.length === 1 ? "" : "s"}</span>
              <Check label="Dropped" checked={dropped} onChange={setDropped} />
            </div>
          )}
          <div className="row">
            <Select value={arrange} onChange={setArrange} options={[{ value: "square", label: "Square" }, { value: "horizontal", label: "Rows" }, { value: "vertical", label: "Columns" }]} />
            <Check label="Overwrite occupied tiles" checked={overwrite} onChange={setOverwrite} />
          </div>
          <Button variant="primary" icon={<Sparkles />} disabled={!origin} onClick={() => {
            if (!origin) return;
            if (src === "item" && spawnItem.id === NONE_ID) { toast("info", "Choose an item to spawn"); return; }
            if (src === "list" && !list.length) { toast("info", "Paste a list of items first"); return; }
            const source = src === "item" ? { type: "item" as const, item: spawnItem, count } : src === "diy" ? { type: "diy" as const, ...diy } : { type: "list" as const, items: list, dropped };
            bulk("Spawn", layer, () => api.fieldSpawn(layer, origin[0], origin[1], arrange, source, overwrite));
          }}>Spawn {origin ? `at X ${origin[0]} Y ${origin[1]}` : ""}</Button>
          {!origin && <div className="tiny faint">Select a tile or an area first: items are laid out from its top-left corner.</div>}
          {pasting && <PasteListModal onClose={() => setPasting(false)} onAdd={(l) => { setList(l); setPasting(false); }} />}
        </section>
        <section className="col">
          <div className="label">Clean up</div>
          <Button icon={<Grid3x3 />} onClick={() => bulk("Remove orphaned extension tiles", layer, () => api.fieldCleanExtensions(layer))}>Remove orphaned extension tiles</Button>
          <Button onClick={async () => {
            const f = await api.fieldUnsupported();
            setFloating(f); setLayer(1);
            toast(f.length ? "warn" : "ok", f.length ? `${f.length} floating tile${f.length === 1 ? "" : "s"} on layer 2` : "No floating items on layer 2", f.length ? "They are outlined in red. Items on layer 2 need something under them on layer 1." : undefined);
          }}>Find floating items on layer 2</Button>
          {floating.length > 0 && <Button size="sm" variant="ghost" onClick={() => setFloating([])}>Hide highlights</Button>}
        </section>
      </div>
    );
  }

  function renderBuildings() {
    const list = buildings!.map((b, i) => ({ b, i })).filter(({ b }) => showEmpty || b.kind);
    const b = selBuilding !== null ? buildings![selBuilding] : null;
    const upd = (patch: Partial<Building>) => {
      if (selBuilding === null) return;
      const next = buildings!.slice(); next[selBuilding] = { ...next[selBuilding], ...patch };
      setBuildings(next); api.setBuildings(next).then(touch).catch((e) => toast("error", "Could not save buildings", String(e)));
    };
    return (
      <div className="col" style={{ gap: 14 }}>
        {b && selBuilding !== null && (
          <div className="col" style={{ gap: 10, padding: 12, border: "1px solid var(--border)", borderRadius: 12, background: "var(--card-2)" }}>
            <div className="row"><h3 style={{ flex: 1 }}>{buildingName(b.kind)}</h3><span className="tiny faint">#{selBuilding}</span></div>
            <Field label="Type"><Select value={b.kind} onChange={(kind) => upd({ kind })} options={enumOptions(gd, "BuildingType")} /></Field>
            <div className="grid cols-2" style={{ gap: 10 }}>
              <Field label="X"><NumberInput value={b.x} max={0xffff} onChange={(x) => upd({ x })} /></Field>
              <Field label="Y"><NumberInput value={b.y} max={0xffff} onChange={(y) => upd({ y })} /></Field>
            </div>
            <Button size="sm" icon={<Crosshair />} onClick={() => setPick({ kind: "building", index: selBuilding })}>Pick position on the map</Button>
            {settings?.advanced && (
              <div className="grid cols-2" style={{ gap: 10 }}>
                <Field label="Angle"><NumberInput value={b.angle} max={255} onChange={(angle) => upd({ angle })} /></Field>
                <Field label="Bit"><NumberInput value={b.bit} max={255} onChange={(bit) => upd({ bit })} /></Field>
                <Field label="Type value"><NumberInput value={b.type_} max={0xffff} onChange={(type_) => upd({ type_ })} /></Field>
                <Field label="Type argument"><NumberInput value={b.type_arg} max={0xffff} onChange={(type_arg) => upd({ type_arg })} /></Field>
                <Field label="Unique id"><NumberInput value={b.unique_id} max={0xffff} onChange={(unique_id) => upd({ unique_id })} /></Field>
              </div>
            )}
            <Callout kind="warn">Moving buildings does not move their footprint in the terrain. Keep the area flat and clear, or the game may misbehave.</Callout>
          </div>
        )}
        <div className="row"><div className="label" style={{ flex: 1 }}>All buildings</div><Check label="Show empty slots" checked={showEmpty} onChange={setShowEmpty} /></div>
        <div className="list">
          {list.map(({ b, i }) => (
            <div key={i} className={"list-item " + (i === selBuilding ? "sel" : "")} onClick={() => {
              setSelBuilding(i);
              if (b.kind && wrap.current) { const r = wrap.current.getBoundingClientRect(); setView((v) => ({ ...v, x: r.width / 2 - b.x * v.s, y: r.height / 2 - b.y * v.s })); }
            }}>
              <Building2 size={15} style={{ color: "var(--text-3)" }} />
              <div style={{ flex: 1 }} className="ellipsis">{b.kind ? buildingName(b.kind) : <span className="faint">Empty slot</span>}</div>
              <span className="tiny faint mono">{b.x}, {b.y}</span>
            </div>
          ))}
        </div>
      </div>
    );
  }

  function renderIsland() {
    const m = meta!;
    const setAcre = (i: number, v: number) => { const acres = m.acres.slice(); acres[i] = v; setMeta({ ...m, acres }); api.setAcres(acres).then(() => { touch(); loadTerrain(); }); };
    return (
      <div className="col" style={{ gap: 16 }}>
        <section className="col">
          <div className="label">Resident Services plaza</div>
          <div className="grid cols-2" style={{ gap: 10 }}>
            <Field label="X"><NumberInput value={m.plaza.x} onChange={(x) => { const plaza = { ...m.plaza, x }; setMeta({ ...m, plaza }); api.setPlaza(plaza).then(touch); }} /></Field>
            <Field label="Z"><NumberInput value={m.plaza.z} onChange={(z) => { const plaza = { ...m.plaza, z }; setMeta({ ...m, plaza }); api.setPlaza(plaza).then(touch); }} /></Field>
          </div>
          <Button size="sm" icon={<Crosshair />} onClick={() => setPick({ kind: "plaza" })}>Pick position on the map</Button>
        </section>
        <section className="col">
          <div className="label">Acre templates</div>
          <div className="small faint">The template of each acre (beaches, rocks, river mouths, the airport…). Only change these if you know what you're doing.</div>
          {settings?.advanced ? (
            <div style={{ display: "grid", gridTemplateColumns: "repeat(9, 1fr)", gap: 3 }}>
              {m.acres.map((a, i) => (
                <input key={`${i}-${a}`} className="input mono" style={{ padding: "0 2px", height: 26, fontSize: 10, textAlign: "center" }} title={`${String.fromCharCode(65 + (i % 9))}${Math.floor(i / 9) + 1}: ${(gd.enums.OutsideAcre ?? []).find(([, v]) => v === a)?.[0] ?? a}`}
                  defaultValue={hex(a, 3)} onBlur={(e) => { const v = parseInt(e.target.value, 16); if (!Number.isNaN(v) && v !== a && v >= 0 && v <= 0xffff) setAcre(i, v); else e.target.value = hex(a, 3); }} />
              ))}
            </div>
          ) : <Callout kind="info">Turn on <b>Advanced mode</b> in Settings to edit acre templates.</Callout>}
        </section>
        <section className="col">
          <div className="label">Ground designs</div>
          <div className="small faint">{m.design_tiles.filter((t) => t !== 0xf800).length} tiles have a custom design painted on the ground ({m.design_dims[0]}×{m.design_dims[1]} grid). Import or export them from the Files tab.</div>
          <Button size="sm" variant="danger" icon={<Trash2 />} onClick={async () => {
            if (!(await confirm({ title: "Remove every ground design?", message: "Custom designs painted on the ground are erased (the designs themselves stay).", danger: true, confirm: "Remove" }))) return;
            const tiles = m.design_tiles.map(() => 0xf800); setMeta({ ...m, design_tiles: tiles }); await api.setDesignTiles(tiles); touch(); toast("ok", "Ground designs removed");
          }}>Remove all ground designs</Button>
        </section>
      </div>
    );
  }

  function renderDumps() {
    const [ax, ay] = acreSel;
    const setAx = (x: number) => setAcreSel([x, ay]), setAy = (y: number) => setAcreSel([ax, y]);
    const reloadAll = async () => {
      const [m, b] = await Promise.all([api.mapMeta(), api.buildings(), loadLayer(0), loadLayer(1), loadTerrain()]);
      setMeta(m); setBuildings(b); changed(0); changed(1); undo.current = []; redo.current = [];
    };
    const rows: { kind: DumpKind; label: string; desc: string; filter: { name: string; extensions: string[] }; name: string; acre?: "field" | "terrain"; perLayer?: boolean }[] = [
      { kind: "field_layer", label: "Field items", desc: `Every item on layer ${layer + 1}`, filter: F.nhl, name: `layer${layer + 1}.nhl`, perLayer: true },
      { kind: "field_acre", label: "Field items (one acre)", desc: "32×32 tiles of the chosen acre", filter: F.nhl, name: `acre-${ax}-${ay}.nhl`, acre: "field", perLayer: true },
      { kind: "field_flags", label: "Item activity flags", desc: `Layer ${layer + 1}`, filter: F.nhlf, name: `layer${layer + 1}.nhlf`, perLayer: true },
      { kind: "terrain", label: "Terrain", desc: "Cliffs, rivers, paths", filter: F.nht, name: "terrain.nht" },
      { kind: "terrain_acre", label: "Terrain (one acre)", desc: "16×16 terrain tiles", filter: F.nht, name: `terrain-${ax}-${ay}.nht`, acre: "terrain" },
      { kind: "buildings", label: "Buildings", desc: "Positions of every building", filter: F.nhb, name: "buildings.nhb" },
      { kind: "acres", label: "Acre templates", desc: "Beaches and outer acres", filter: F.nha, name: "acres.nha" },
      { kind: "designs", label: "Ground designs", desc: "Custom designs on the ground", filter: F.nhmd, name: "designs.nhmd" },
    ];
    return (
      <div className="col" style={{ gap: 12 }}>
        <div className="small faint">Share or back up parts of the island. Files are compatible with the original NHSE.</div>
        <div className="row">
          <Field label="Acre column"><NumberInput value={ax} max={8} onChange={setAx} width={70} /></Field>
          <Field label="Acre row"><NumberInput value={ay} max={6} onChange={setAy} width={70} /></Field>
          <div className="tiny faint" style={{ flex: 1 }}>Used by the one-acre files (counted from the top-left of each layer).</div>
        </div>
        {rows.map((r) => (
          <div key={r.kind} className="row" style={{ padding: "8px 10px", border: "1px solid var(--border)", borderRadius: 10 }}>
            <div style={{ flex: 1, minWidth: 0 }}><div className="bold">{r.label}</div><div className="tiny faint">{r.desc}</div></div>
            <Button size="sm" iconOnly icon={<Download />} title="Export" onClick={async () => {
              const p = await saveFile(`Export ${r.label.toLowerCase()}`, r.name, [r.filter]);
              if (!p) return;
              await syncChain.current;
              try { await api.mapExport(r.kind, p, layer, r.acre ? ax : undefined, r.acre ? ay : undefined); toast("ok", `${r.label} exported`); } catch (e) { toast("error", "Export failed", String(e)); }
            }} />
            <Button size="sm" iconOnly icon={<Upload />} title="Import" onClick={async () => {
              const p = await pickFile(`Import ${r.label.toLowerCase()}`, [r.filter, F.all]);
              if (!p) return;
              await syncChain.current;
              try { await api.mapImport(r.kind, p, layer, r.acre ? ax : undefined, r.acre ? ay : undefined); await reloadAll(); touch(); toast("ok", `${r.label} imported`); } catch (e) { toast("error", "Import failed", String(e)); }
            }} />
          </div>
        ))}
      </div>
    );
  }
}
