import { Brush, Download, Eraser, Home, MousePointer2, RotateCw, Trash2, Upload } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { FlagTable } from "../components/FlagTable";
import { ItemEditor } from "../components/ItemEditor";
import { ItemPicker } from "../components/ItemPicker";
import { Badge, Button, Callout, Card, Field, Loading, NumberInput, Segmented, Select, Tabs, ToggleRow } from "../components/ui";
import { enumOptions, prettify } from "../components/Widgets";
import { api, type HouseExterior, type Item, type PlayerHouse, type PlayerRoom, type RoomFloorWall, type VillagerHouse } from "../lib/api";
import { F, pickFile, saveFile } from "../lib/dialog";
import { Grid, NONE_ITEM } from "../lib/grid";
import { EXT_ID, NONE_ID, footprint, itemColor, itemLabel, itemSprite, variantOf } from "../lib/items";
import { useGD, useLoad, useStore } from "../lib/store";

const ROOM_NAMES = ["Room 1 (entrance)", "Room 2", "Room 3", "Room 4", "Room 5", "Room 6"];

export default function Houses() {
  const [tab, setTab] = useState<"players" | "villagers">("players");
  return (
    <div className="content"><div className="page wide">
      <Tabs value={tab} onChange={setTab} tabs={[{ value: "players", label: "Player houses", icon: <Home /> }, { value: "villagers", label: "Villager houses", icon: <Home /> }]} />
      {tab === "players" ? <PlayerHouses /> : <VillagerHouses />}
    </div></div>
  );
}

function ExteriorEditor({ e, onChange, extra }: { e: HouseExterior; onChange: (e: HouseExterior) => void; extra?: React.ReactNode }) {
  const gd = useGD();
  const set = (p: Partial<HouseExterior>) => onChange({ ...e, ...p });
  return (
    <div className="col" style={{ gap: 12 }}>
      <div className="grid cols-4" style={{ gap: 12 }}>
        <Field label="Walls"><Select value={e.wall} onChange={(x) => set({ wall: x, order_wall: x })} options={enumOptions(gd, "WallType")} /></Field>
        <Field label="Roof"><Select value={e.roof} onChange={(x) => set({ roof: x, order_roof: x })} options={enumOptions(gd, "RoofType")} /></Field>
        <Field label="Door"><Select value={e.door} onChange={(x) => set({ door: x, order_door: x })} options={enumOptions(gd, "DoorKind")} /></Field>
        <Field label="House level"><NumberInput value={e.level} max={255} onChange={(x) => set({ level: x })} /></Field>
      </div>
      <div className="grid cols-2" style={{ gap: 12 }}>
        <div className="col" style={{ gap: 6 }}><div className="label">Door decoration</div><ItemEditor item={e.door_deco} compact allowWrap={false} onChange={(it) => set({ door_deco: it })} /></div>
        {extra}
      </div>
    </div>
  );
}

function PlayerHouses() {
  const { touch, toast, save, settings } = useStore();
  const [houses, setHouses] = useLoad(api.playerHouses, []);
  const [sel, setSel] = useState(0);
  const [roomIdx, setRoomIdx] = useState(0);
  if (!houses) return <Loading />;
  if (!houses.length) return <Callout kind="info">No player houses were found in this save.</Callout>;
  const h = houses[Math.min(sel, houses.length - 1)];
  const owner = (i: number) => save?.players[i]?.name ?? `Player ${i + 1}`;
  const upd = (next: PlayerHouse) => {
    setHouses(houses.map((x) => (x.index === next.index ? next : x)));
    api.setPlayerHouse(next).then(touch).catch((e) => toast("error", "Could not save house", String(e)));
  };
  return (
    <>
      <div className="row wrap">
        {houses.map((x) => (
          <button key={x.index} className={"chip " + (x.index === h.index ? "on" : "")} style={x.index === h.index ? { borderColor: "var(--accent)", color: "var(--accent-2)" } : undefined} onClick={() => { setSel(x.index); setRoomIdx(0); }}>
            <Home size={13} /> {owner(x.index)}
          </button>
        ))}
      </div>
      <Card title={`${owner(h.index)}'s house`} desc={`House #${h.index + 1}`} actions={<>
        <Button size="sm" icon={<Download />} onClick={async () => { const p = await saveFile("Export house", `${owner(h.index)}-house.nhph${h.format >= 2 ? "2" : ""}`, [F.nhph]); if (p) { await api.exportPlayerHouse(h.index, p); toast("ok", "House exported"); } }}>Export</Button>
        <Button size="sm" icon={<Upload />} onClick={async () => { const p = await pickFile("Import house", [F.nhph, F.all]); if (!p) return; try { await api.importPlayerHouse(h.index, p); setHouses(await api.playerHouses()); touch(); toast("ok", "House imported"); } catch (e) { toast("error", "Import failed", String(e)); } }}>Import</Button>
      </>}>
        <ExteriorEditor e={h.exterior} onChange={(exterior) => upd({ ...h, exterior })} extra={
          <div className="col" style={{ gap: 6 }}><div className="label">Mailbox</div><ItemEditor item={h.mailbox} compact allowWrap={false} onChange={(it) => upd({ ...h, mailbox: it, order_mailbox: it })} /></div>
        } />
        <div className="grid cols-3" style={{ gap: 12, marginTop: 12 }}>
          <Field label="Home loan (Bells)"><NumberInput value={h.loan} onChange={(loan) => upd({ ...h, loan })} suffix="Bells" /></Field>
          <ToggleRow title="House built" desc="Marks the house as built (not a tent)." on={h.player_house_flag} onChange={(v) => upd({ ...h, player_house_flag: v })} />
        </div>
        {settings?.advanced && <div style={{ marginTop: 14 }}><div className="label" style={{ marginBottom: 6 }}>House event flags</div><FlagTable values={h.event_flags} defs={[]} height={260} onChange={(f) => upd({ ...h, event_flags: f })} /></div>}
      </Card>
      <Card bodyClass="">
        <div style={{ padding: "8px 10px 0" }}>
          <Tabs value={String(roomIdx)} onChange={(v) => setRoomIdx(Number(v))} tabs={ROOM_NAMES.map((n, i) => ({ value: String(i), label: n }))} />
        </div>
        <div className="card-body"><RoomEditor key={`${h.index}-${roomIdx}`} house={h.index} room={roomIdx} /></div>
      </Card>
    </>
  );
}

type Tool = "select" | "place" | "erase";

function RoomEditor({ house, room }: { house: number; room: number }) {
  const gd = useGD();
  const { touch, toast, settings } = useStore();
  const [data, setData] = useState<PlayerRoom | null>(null);
  const [layer, setLayer] = useState(0);
  const [tool, setTool] = useState<Tool>("select");
  const [selTile, setSelTile] = useState<[number, number] | null>(null);
  const [brush, setBrush] = useState<Item>({ ...NONE_ITEM });
  const [hover, setHover] = useState<[number, number] | null>(null);
  const cell = 28;
  useEffect(() => { api.playerRoom(house, room).then(setData).catch((e) => toast("error", "Could not read room", String(e))); }, [house, room]);
  const grid = useMemo(() => (data ? new Grid(20, 20, data.layers[layer]) : null), [data, layer]);
  if (!data || !grid) return <Loading />;
  const overhang = !!settings?.room_allow_overhang;

  const commit = (next: PlayerRoom) => {
    setData(next);
    api.setPlayerRoom(house, room, next).then(touch).catch((e) => toast("error", "Could not save room", String(e)));
  };
  const commitGrid = (g: Grid) => { const layers = data.layers.slice(); layers[layer] = g.tiles; commit({ ...data, layers }); };

  const click = (x: number, y: number) => {
    if (tool === "select") { setSelTile(grid.rootOf(x, y) ?? [x, y]); return; }
    const g = grid.clone();
    if (tool === "erase") { if (g.removeAt(gd, x, y)) commitGrid(g); return; }
    if (brush.id === NONE_ID) { toast("info", "Pick an item to place first"); return; }
    const r = g.check(gd, brush, x, y, overhang);
    if (r === "bounds") { toast("warn", "Doesn't fit here", overhang ? undefined : "Enable “allow items to overhang” in Settings to place it partly outside the grid."); return; }
    if (r === "collision") { toast("warn", "That spot is occupied"); return; }
    g.place(gd, brush, x, y, true);
    commitGrid(g);
    setSelTile([x, y]);
  };

  const roots: { x: number; y: number; it: Item; w: number; h: number }[] = [];
  const dangling: [number, number][] = [];
  for (let x = 0; x < 20; x++) for (let y = 0; y < 20; y++) {
    const it = grid.get(x, y);
    if (it.id === NONE_ID) continue;
    if (it.id === EXT_ID) { if (!grid.rootOf(x, y)) dangling.push([x, y]); continue; }
    const [w, h] = footprint(gd, it);
    roots.push({ x, y, it, w, h });
  }
  const selItem = selTile ? grid.get(selTile[0], selTile[1]) : null;
  const ghost = tool === "place" && hover && brush.id !== NONE_ID ? (() => { const [w, h] = footprint(gd, brush); return { x: hover[0], y: hover[1], w, h, ok: grid.check(gd, brush, hover[0], hover[1], overhang) === "ok" }; })() : null;
  const surf = (k: keyof RoomFloorWall) => (it: Item) => commit({ ...data, surfaces: { ...data.surfaces, [k]: it } });

  return (
    <div className="grid" style={{ gridTemplateColumns: `${20 * cell + 2}px minmax(280px, 1fr)`, gap: 20, alignItems: "start" }}>
      <div className="col" style={{ gap: 10 }}>
        <div className="row wrap">
          <Segmented value={tool} onChange={setTool} options={[
            { value: "select", label: "Select", icon: <MousePointer2 /> }, { value: "place", label: "Place", icon: <Brush /> }, { value: "erase", label: "Erase", icon: <Eraser /> }]} />
          <div style={{ flex: 1 }} />
          <Select value={layer} onChange={(l) => { setLayer(l); setSelTile(null); }} options={gd.enums.RoomLayerSurface.map(([n, v]) => ({ value: v, label: prettify(n) }))} />
        </div>
        <div style={{ position: "relative", width: 20 * cell + 2, height: 20 * cell + 2, borderRadius: 10, border: "1px solid var(--border-strong)", overflow: "hidden",
          background: `repeating-conic-gradient(var(--card-2) 0 25%, var(--card) 0 50%) 0 0 / ${cell * 2}px ${cell * 2}px` }}
          onMouseLeave={() => setHover(null)}>
          {Array.from({ length: 400 }, (_, i) => {
            const x = Math.floor(i / 20), y = i % 20;
            return <div key={i} onMouseEnter={() => setHover([x, y])} onMouseDown={() => click(x, y)}
              style={{ position: "absolute", left: x * cell, top: y * cell, width: cell, height: cell, boxShadow: "inset 0 0 0 0.5px var(--border)", cursor: tool === "select" ? "default" : "crosshair" }} />;
          })}
          {roots.map(({ x, y, it, w, h }) => {
            const sel = selTile && selTile[0] === x && selTile[1] === y;
            return (
              <div key={`${x}-${y}`} title={itemLabel(gd, it)} style={{ position: "absolute", left: x * cell + 1, top: y * cell + 1, width: w * cell - 2, height: h * cell - 2, pointerEvents: "none",
                borderRadius: 6, background: itemColor(gd, it.id) + "55", border: `1.5px solid ${sel ? "var(--accent)" : itemColor(gd, it.id)}`, boxShadow: sel ? "0 0 0 3px var(--accent-soft)" : undefined,
                display: "grid", placeItems: "center" }}>
                <img src={itemSprite(it.id, variantOf(it))} alt="" style={{ width: Math.min(w, h) * cell * 0.85, height: Math.min(w, h) * cell * 0.85, objectFit: "contain", transform: `rotate(${(it.system & 3) * 90}deg)` }} onError={(e) => (e.currentTarget.style.visibility = "hidden")} />
              </div>
            );
          })}
          {dangling.map(([x, y]) => <div key={`d${x}-${y}`} title="Orphaned extension tile" style={{ position: "absolute", left: x * cell, top: y * cell, width: cell, height: cell, pointerEvents: "none", background: "repeating-linear-gradient(45deg, var(--danger-soft) 0 4px, transparent 4px 8px)" }} />)}
          {ghost && <div style={{ position: "absolute", left: ghost.x * cell, top: ghost.y * cell, width: ghost.w * cell, height: ghost.h * cell, pointerEvents: "none", borderRadius: 6,
            border: `2px dashed ${ghost.ok ? "var(--accent)" : "var(--danger)"}`, background: ghost.ok ? "var(--accent-soft)" : "var(--danger-soft)" }} />}
        </div>
        <div className="row small faint">
          {hover ? <>Tile {hover[0]}, {hover[1]} · {itemLabel(gd, grid.get(hover[0], hover[1]))}</> : <>{roots.length} item{roots.length === 1 ? "" : "s"} on this layer</>}
          <div style={{ flex: 1 }} />
          {dangling.length > 0 && <Badge kind="warn">{dangling.length} orphaned tiles</Badge>}
          <Button size="sm" variant="ghost" icon={<Trash2 />} onClick={() => { commitGrid(new Grid(20, 20, grid.tiles.map(() => ({ ...NONE_ITEM })))); setSelTile(null); }}>Clear layer</Button>
        </div>
      </div>
      <div className="col" style={{ gap: 16 }}>
        {tool === "place" && (
          <div className="col" style={{ gap: 6 }}>
            <div className="label">Item to place</div>
            <ItemEditor item={brush} allowWrap={false} onChange={setBrush} />
            <Button size="sm" icon={<RotateCw />} onClick={() => setBrush({ ...brush, system: (brush.system & ~3) | (((brush.system & 3) + 1) & 3) })}>Rotate</Button>
          </div>
        )}
        {tool === "select" && selTile && selItem && selItem.id !== NONE_ID && (
          <div className="col" style={{ gap: 6 }}>
            <div className="label">Selected item · tile {selTile[0]}, {selTile[1]}</div>
            <ItemEditor item={selItem} allowWrap={false} onChange={(it) => {
              const g = grid.clone();
              g.removeAt(gd, selTile[0], selTile[1]);
              if (it.id !== NONE_ID && g.check(gd, it, selTile[0], selTile[1], overhang) !== "ok") { toast("warn", "The changed item no longer fits here"); return; }
              if (it.id !== NONE_ID) g.place(gd, it, selTile[0], selTile[1], true);
              commitGrid(g);
            }} />
            <div className="row">
              <Button size="sm" variant="danger" icon={<Trash2 />} onClick={() => { const g = grid.clone(); g.removeAt(gd, selTile[0], selTile[1]); commitGrid(g); setSelTile(null); }}>Remove</Button>
              <Button size="sm" icon={<Brush />} onClick={() => { setBrush(selItem); setTool("place"); }}>Use as brush</Button>
            </div>
          </div>
        )}
        {tool === "select" && (!selItem || selItem.id === NONE_ID) && <div className="faint small">Click an item in the room to edit it, or switch to <b>Place</b> to add furniture.</div>}
        <div className="divider" />
        <div className="label">Wallpaper, flooring & rug</div>
        {(["wall", "floor", "accent_wall"] as const).map((k) => (
          <Field key={k} label={k === "wall" ? "Wallpaper" : k === "floor" ? "Flooring" : "Accent wall"}>
            <ItemEditor item={data.surfaces[k]} compact allowWrap={false} onChange={surf(k)} />
          </Field>
        ))}
        <div className="divider" />
        <Field label="Music">
          <ItemPicker value={data.music < 0 ? NONE_ID : data.music} placeholder="No music" filter={(e) => gd.kind_names[gd.kinds[e.id]] === "Kind_Music"}
            onChange={(id) => commit({ ...data, music: id === NONE_ID ? -1 : id })} />
        </Field>
        <div className="row">
          <ToggleRow title="Shuffle music" on={data.music_shuffle} onChange={(v) => commit({ ...data, music_shuffle: v })} />
        </div>
        {data.ambient !== null && (
          <Field label="Ambient sound"><Select value={data.ambient} onChange={(a) => commit({ ...data, ambient: a })} options={enumOptions(gd, "SoundAmbientKind")} /></Field>
        )}
        <div className="row">
          <Button size="sm" icon={<Download />} onClick={async () => { const p = await saveFile("Export room", `room-${room + 1}.nhpr`, [F.nhpr]); if (p) { await api.exportPlayerRoom(house, room, p); toast("ok", "Room exported"); } }}>Export room</Button>
          <Button size="sm" icon={<Upload />} onClick={async () => { const p = await pickFile("Import room", [F.nhpr, F.all]); if (!p) return; try { await api.importPlayerRoom(house, room, p); setData(await api.playerRoom(house, room)); touch(); toast("ok", "Room imported"); } catch (e) { toast("error", "Import failed", String(e)); } }}>Import room</Button>
        </div>
      </div>
    </div>
  );
}

function VillagerHouses() {
  const gd = useGD();
  const { touch, toast } = useStore();
  const [houses, setHouses] = useLoad(api.villagerHouses, []);
  const [villagers] = useLoad(api.villagers, []);
  const [sel, setSel] = useState(0);
  if (!houses || !villagers) return <Loading />;
  const h = houses[Math.min(sel, houses.length - 1)];
  const who = (x: VillagerHouse) => {
    const v = villagers.find((v) => v.index === x.npc1);
    if (!v || v.species === 35) return "Unoccupied";
    return gd.villagers.find((g) => g.internal === v.internal_name)?.name ?? v.internal_name;
  };
  const upd = (next: VillagerHouse) => { setHouses(houses.map((x) => (x.index === next.index ? next : x))); api.setVillagerHouse(next).then(touch).catch((e) => toast("error", "Could not save house", String(e))); };
  return (
    <>
      <div className="row wrap">
        {houses.map((x) => (
          <button key={x.index} className="chip" style={x.index === h.index ? { borderColor: "var(--accent)", color: "var(--accent-2)" } : undefined} onClick={() => setSel(x.index)}>
            <Home size={13} /> {who(x)}
          </button>
        ))}
      </div>
      <Card title={`${who(h)}'s house`} desc={`House #${h.index + 1}`} actions={<>
        <Button size="sm" icon={<Download />} onClick={async () => { const p = await saveFile("Export house", `villager-house-${h.index + 1}.nhvh${h.format >= 2 ? "2" : ""}`, [F.nhvh]); if (p) { await api.exportVillagerHouse(h.index, p); toast("ok", "House exported"); } }}>Export</Button>
        <Button size="sm" icon={<Upload />} onClick={async () => { const p = await pickFile("Import house", [F.nhvh, F.all]); if (!p) return; try { await api.importVillagerHouse(h.index, p); setHouses(await api.villagerHouses()); touch(); toast("ok", "House imported"); } catch (e) { toast("error", "Import failed", String(e)); } }}>Import</Button>
      </>}>
        <ExteriorEditor e={h.exterior} onChange={(exterior) => upd({ ...h, exterior })} />
        {h.room && (
          <>
            <div className="divider" style={{ margin: "14px 0" }} />
            <div className="grid cols-3" style={{ gap: 12 }}>
              {(["wall", "floor", "accent_wall"] as const).map((k) => (
                <Field key={k} label={k === "wall" ? "Wallpaper" : k === "floor" ? "Flooring" : "Accent wall"}>
                  <ItemEditor item={h.room![k]} compact allowWrap={false} onChange={(it) => upd({ ...h, room: { ...h.room!, [k]: it } })} />
                </Field>
              ))}
            </div>
          </>
        )}
      </Card>
    </>
  );
}
