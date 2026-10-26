import { Droplets, Gift, Sparkles } from "lucide-react";
import React, { useEffect, useMemo, useState } from "react";
import { api, type Item, type ItemInfo } from "../lib/api";
import { baseName, buildIndex, count, hex, itemSprite, NONE_ID, type SearchEntry } from "../lib/items";
import { useStore } from "../lib/store";
import { ItemPicker } from "./ItemPicker";
import { isStackable, STACK_MAX } from "./ItemSlot";
import { Check, Field, NumberInput, Select, Switch } from "./ui";

const infoCache = new Map<number, ItemInfo>();
export function useItemInfo(id: number): ItemInfo | null {
  const [info, setInfo] = useState<ItemInfo | null>(infoCache.get(id) ?? null);
  useEffect(() => {
    if (id === NONE_ID) { setInfo(null); return; }
    const c = infoCache.get(id);
    if (c) { setInfo(c); return; }
    let live = true;
    api.itemInfo(id).then((i) => { infoCache.set(id, i); if (live) setInfo(i); }).catch(() => {});
    return () => { live = false; };
  }, [id]);
  return info;
}
export const clearItemInfoCache = () => infoCache.clear();

const WRAP_PAPERS = ["Yellow", "Pink", "Orange", "Light green", "Green", "Mint", "Light blue", "Purple", "Navy", "Blue", "White", "Red", "Gold", "Brown", "Gray", "Black"];
const WRAP_TYPES = ["None", "Wrapping paper", "Present", "Delivery box", "Festive"];

const GENES: [string, number, boolean][] = [["R", 0, false], ["R", 1, false], ["Y", 2, false], ["Y", 3, false], ["W", 4, true], ["W", 5, true], ["S", 6, false], ["S", 7, false]];

/** Edits every property of an item. */
export function ItemEditor({ item, onChange, allowWrap = true, compact }: {
  item: Item; onChange: (it: Item) => void; allowWrap?: boolean; compact?: boolean;
}) {
  const { gd, settings } = useStore();
  const isExt = item.id === 0xfffd;
  const id = isExt ? item.free & 0xffff : item.id;
  const info = useItemInfo(id);
  const set = (patch: Partial<Item>) => onChange({ ...item, ...patch });
  const free = item.free >>> 0;
  const setFree = (f: number) => set({ free: f >>> 0 });
  const kindN = gd && id < gd.kinds.length ? gd.kind_names[gd.kinds[id]] : "";
  const stackable = gd && id < gd.kinds.length && isStackable(gd, id);
  const maxStack = info?.max_stack ?? STACK_MAX[kindN] ?? 1;

  const recipeEntries = useMemo<SearchEntry[]>(() => gd ? gd.recipes.map(([r, it]) => ({ id: r, name: `${baseName(gd, it)}`, key: baseName(gd, it).toLowerCase() })) : [], [gd]);
  const fossilEntries = useMemo<SearchEntry[]>(() => gd ? (gd.lists.Fossils ?? []).map((f) => ({ id: f, name: baseName(gd, f), key: baseName(gd, f).toLowerCase() })) : [], [gd]);
  const treeDrops = useMemo<SearchEntry[]>(() => {
    if (!gd) return [];
    const ids = [...(gd.lists.Fruits ?? [])];
    const uniq = [...new Set(ids)].filter((i) => i < gd.items.length);
    const all = buildIndex(gd, false, false);
    const bells = all.filter((e) => /^[\d,.]+ bells$/i.test(e.name)).slice(0, 12);
    return [{ id: 0, name: "(nothing)", key: "nothing" }, ...uniq.map((i) => ({ id: i, name: gd.items[i], key: gd.items[i].toLowerCase() })), ...bells];
  }, [gd]);

  if (!gd) return null;

  const pickItem = (nid: number) => {
    if (isExt) { set({ free: (free & 0xffff0000) | nid }); return; }
    // reset parameters that do not carry over between item types
    onChange({ id: nid, system: item.system & 0x03, additional: 0, free: 0 });
  };

  const genes = free & 0xff;
  const watered = (free >>> 8) & 0xffff;
  const setWatered = (w: number) => setFree((free & 0xff0000ff) | ((w & 0xffff) << 8));

  return (
    <div className="col" style={{ gap: 12 }}>
      <div className="row" style={{ gap: 12, alignItems: "flex-start" }}>
        <div className="slot" style={{ "--slot": compact ? "52px" : "64px" } as React.CSSProperties}>
          {id !== NONE_ID && <img src={itemSprite(id, free & 7)} alt="" />}
        </div>
        <div className="col" style={{ flex: 1, gap: 6, minWidth: 0 }}>
          <ItemPicker value={id} onChange={pickItem} />
          <div className="row small faint" style={{ gap: 10, flexWrap: "wrap" }}>
            {id !== NONE_ID && <span className="mono">{hex(id)}</span>}
            {info && <span>{info.kind_name.replace(/^Kind_/, "")}</span>}
            {info && <span>{info.width / 2}×{info.height / 2}</span>}
            {isExt && <span className="badge info">extension tile</span>}
          </div>
        </div>
      </div>

      {id !== NONE_ID && !isExt && (
        <>
          {info?.is_flower_gene ? (
            <div className="col" style={{ gap: 10 }}>
              <div className="label"><Sparkles size={14} /> Genes</div>
              <div className="row wrap" style={{ gap: 6 }}>
                {GENES.map(([g, bit, inv], i) => {
                  const set1 = ((genes >> bit) & 1) === 1;
                  const on = inv ? !set1 : set1;
                  return (
                    <button key={i} className={"chip " + (on ? "on" : "")} title={inv ? "White gene (stored inverted)" : undefined}
                      onClick={() => setFree((free & ~0xff) | (genes ^ (1 << bit)))}>
                      {on ? g : g.toLowerCase()}<sub>{(bit % 2) + 1}</sub>
                    </button>
                  );
                })}
              </div>
              <div className="label"><Droplets size={14} /> Watering</div>
              <div className="row wrap" style={{ gap: 12 }}>
                <Field label="Days watered"><NumberInput value={watered & 0x1f} min={0} max={31} onChange={(v) => setWatered((watered & 0xffe0) | v)} width={80} /></Field>
                <Check label="Watered today" checked={(watered & 0x8000) !== 0} onChange={(v) => setWatered((watered & 0x7fff) | (v ? 0x8000 : 0))} />
                <Check label="Gold can" checked={((free >>> 24) & 1) === 1} onChange={(v) => setFree((free & ~0x01000000) | (v ? 0x01000000 : 0))} />
                <button className="btn sm" onClick={() => setFree((free & 0xff) | 0xffff00 | 0x01000000)}>Water fully</button>
              </div>
              <div className="row wrap" style={{ gap: 4 }}>
                {Array.from({ length: 10 }, (_, v) => (
                  <Check key={v} label={`V${v + 1}`} title={`Watered by visitor ${v + 1}`} checked={(watered & (1 << (5 + v))) !== 0}
                    onChange={(on) => setWatered(on ? watered | (1 << (5 + v)) : watered & ~(1 << (5 + v)))} />
                ))}
              </div>
            </div>
          ) : info?.is_recipe ? (
            <Field label="Recipe">
              <ItemPicker value={free & 0xffff} entries={recipeEntries} placeholder="Choose a recipe" onChange={(r) => setFree((free & 0xffff0000) | r)} />
            </Field>
          ) : info?.is_fossil_unknown ? (
            <Field label="Fossil inside">
              <ItemPicker value={free & 0xffff} entries={fossilEntries} onChange={(f) => setFree((free & 0xffff0000) | f)} />
            </Field>
          ) : info?.is_tree ? (
            <Field label="Fruit / drop" hint="What shakes out of the tree">
              <ItemPicker value={count(item) || NONE_ID} entries={treeDrops} placeholder="(nothing)" onChange={(d) => setFree((free & 0xffff0000) | (d === NONE_ID ? 0 : d))} />
            </Field>
          ) : (
            <div className="row wrap" style={{ gap: 12, alignItems: "flex-end" }}>
              {stackable && maxStack > 1 && (
                <Field label="Quantity" hint={`Stacks up to ${maxStack}`} hintKind={count(item) + 1 > maxStack ? "warn" : undefined}>
                  <div className="row">
                    <NumberInput value={count(item) + 1} min={1} max={0x10000} onChange={(v) => setFree((free & 0xffff0000) | ((v - 1) & 0xffff))} width={90} />
                    <button className="btn sm" onClick={() => setFree((free & 0xffff0000) | (maxStack - 1))}>Max</button>
                  </div>
                </Field>
              )}
              {info && info.body.length > 0 && (
                <Field label={info.body_label || "Variation"}>
                  <Select value={free & 7} onChange={(v) => setFree((free & ~7) | v)} style={{ minWidth: 150 }}
                    options={info.body.map((b) => ({ value: b.index, label: `${b.name || "Variant " + (b.index + 1)}${b.colors ? " · " + b.colors : ""}` }))} />
                </Field>
              )}
              {info && info.fabric_customizable && (
                <Field label={info.fabric_label || "Pattern"}>
                  <Select value={(free >>> 5) & 0x7ff} onChange={(v) => setFree((free & 0x1f) | ((v & 0x7ff) << 5))} style={{ minWidth: 150 }}
                    options={[...(info.fabric.length ? info.fabric : [{ index: 0, name: "Default", colors: "" }]).map((f) => ({ value: f.index, label: `${f.name || "Pattern " + (f.index + 1)}${f.colors ? " · " + f.colors : ""}` }))]} />
                </Field>
              )}
            </div>
          )}

          {allowWrap && !info?.is_field_item && !info?.is_recipe && (
            <div className="col" style={{ gap: 8 }}>
              <div className="row" style={{ gap: 10 }}>
                <div className="label" style={{ flex: 1 }}><Gift size={14} /> Wrapping</div>
                <Switch on={(item.additional & 3) !== 0} onChange={(v) => set({ additional: v ? 1 : 0 })} />
              </div>
              {(item.additional & 3) !== 0 && (
                <div className="row wrap" style={{ gap: 10 }}>
                  <Select value={(item.additional & 3) === 3 && ((item.additional >> 2) & 15) === 15 ? 4 : item.additional & 3}
                    onChange={(t) => {
                      const show = item.additional & 0x40, f80 = item.additional & 0x80;
                      const a = t === 4 ? 3 | (15 << 2) : t === 1 ? 1 | (item.additional & 0x3c) : t;
                      set({ additional: a | show | f80 });
                    }}
                    options={WRAP_TYPES.slice(1).map((l, i) => ({ value: i + 1, label: l }))} style={{ width: 160 }} />
                  {(item.additional & 3) === 1 && (
                    <Select value={(item.additional >> 2) & 15} onChange={(c) => set({ additional: (item.additional & 0xc3) | (c << 2) })}
                      options={WRAP_PAPERS.map((l, i) => ({ value: i, label: l }))} style={{ width: 130 }} />
                  )}
                  <Check label="Show item name" checked={(item.additional & 0x40) !== 0} onChange={(v) => set({ additional: (item.additional & ~0x40) | (v ? 0x40 : 0) })} />
                </div>
              )}
            </div>
          )}
        </>
      )}

      {settings?.advanced && id !== NONE_ID && (
        <details>
          <summary className="label" style={{ cursor: "default" }}>Raw values</summary>
          <div className="grid cols-2" style={{ gap: 8, marginTop: 8 }}>
            <Field label="Item ID (hex)"><NumberInput hex value={item.id} max={0xffff} onChange={(v) => set({ id: v })} /></Field>
            <Field label="System param (hex)" hint="rotation · buried 0x04 · dropped 0x20"><NumberInput hex value={item.system} max={0xff} onChange={(v) => set({ system: v })} /></Field>
            <Field label="Additional param (hex)"><NumberInput hex value={item.additional} max={0xff} onChange={(v) => set({ additional: v })} /></Field>
            <Field label="Free param (hex)"><NumberInput hex value={free} max={0xffffffff} onChange={(v) => setFree(v)} /></Field>
            {!info?.is_flower_gene && <Field label="Uses (durability)"><NumberInput value={free >>> 16} max={0xffff} onChange={(v) => setFree((free & 0xffff) | (v << 16))} /></Field>}
          </div>
        </details>
      )}
    </div>
  );
}
