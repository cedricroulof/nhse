import { BookOpen, Search, ShieldCheck } from "lucide-react";
import { useMemo, useState } from "react";
import { Button, Callout, Card, Check, Loading, MenuButton, Segmented } from "../components/ui";
import { api, itemSet } from "../lib/api";
import { hex, itemSprite } from "../lib/items";
import { useGD, useLoad, useStore } from "../lib/store";

const ROW = 38;
const GROUPS: [string, string][] = [["fish", "Fish"], ["bugs", "Bugs"], ["sea", "Sea creatures"], ["fossils", "Fossils"], ["art", "Art"], ["furniture", "Furniture"],
  ["wearables", "Clothing & accessories"], ["interior", "Wallpaper, floors & rugs"], ["tools", "Tools"], ["music", "Songs"]];

export default function Catalog() {
  const { player, touch, toast, settings } = useStore();
  const gd = useGD();
  const [rec, setRec] = useLoad(() => api.playerReceived(player), [player]);
  const [q, setQ] = useState("");
  const [filter, setFilter] = useState<"all" | "have" | "missing">("all");
  const [variants, setVariants] = useState(true);
  const [scroll, setScroll] = useState(0);
  const rows = useMemo(() => {
    if (!rec) return [];
    const s = q.trim().toLowerCase();
    const out: number[] = [];
    for (let i = 1; i < gd.items.length; i++) {
      if (!gd.named[i] && !settings?.advanced) continue;
      if (s && !gd.items[i].toLowerCase().includes(s) && hex(i).toLowerCase() !== s) continue;
      const has = rec.items[i] === 1;
      if (filter === "have" && !has) continue;
      if (filter === "missing" && has) continue;
      out.push(i);
    }
    return out;
  }, [gd, rec, q, filter, settings?.advanced]);
  // remake index lookup: item -> remake index
  const remakeOf = useMemo(() => {
    const m = new Map<number, number>();
    rec?.remake_items.forEach((item, idx) => { if (item) m.set(item, idx); });
    return m;
  }, [rec]);
  if (!rec) return <Loading />;
  const commit = (items: number[], remake: number[]) => { setRec({ ...rec, items, remake }); api.setPlayerReceived(player, items, remake).then(touch); };
  const setItem = (id: number, v: boolean, items = rec.items.slice(), remake = rec.remake.slice()) => {
    items[id] = v ? 1 : 0;
    const ri = remakeOf.get(id);
    if (ri !== undefined && variants) for (let k = 0; k < 8; k++) if (ri * 8 + k < remake.length) remake[ri * 8 + k] = v ? 1 : 0;
    return [items, remake] as const;
  };
  const give = async (cat: string) => {
    const list = await itemSet(cat, false);
    let items = rec.items.slice(), remake = rec.remake.slice();
    for (const it of list) [items, remake] = setItem(it.id, true, items, remake) as [number[], number[]];
    commit(items, remake);
    toast("ok", `Marked ${list.length} items as collected`);
  };
  const giveSafeEverything = async () => {
    const cats = GROUPS.map(([k]) => k);
    let items = rec.items.slice(), remake = rec.remake.slice(), n = 0;
    for (const c of cats) for (const it of await itemSet(c, false)) { [items, remake] = setItem(it.id, true, items, remake) as [number[], number[]]; n++; }
    commit(items, remake);
    toast("ok", `Marked ${n} catalog items as collected`);
  };
  const H = 560;
  const start = Math.max(0, Math.floor(scroll / ROW) - 6);
  const end = Math.min(rows.length, start + Math.ceil(H / ROW) + 12);
  const have = rec.items.reduce((a, b) => a + b, 0);
  return (
    <div className="content"><div className="page">
      <Callout kind="info" title="Safer bulk unlocks">“Give” actions only use real catalog items. Internal, dummy and event-only entries — which were known to crash Nook Shopping when marked as collected — are excluded.</Callout>
      <Card title={<span className="row"><BookOpen size={16} /> Catalog (received items)</span>} desc={`${have.toLocaleString()} items collected`}
        actions={<div className="row">
          <MenuButton size="sm" label="Give…" items={GROUPS.map(([k, l]) => ({ label: l, onClick: () => give(k) }))} />
          <Button size="sm" variant="primary" icon={<ShieldCheck />} onClick={giveSafeEverything}>Give everything (safe)</Button>
          <Button size="sm" variant="danger" onClick={() => commit(rec.items.map(() => 0), rec.remake.map(() => 0))}>Clear</Button>
        </div>}>
        <div className="row wrap" style={{ marginBottom: 10 }}>
          <div className="input-wrap" style={{ width: 280 }}><Search /><input className="input" placeholder="Search items…" value={q} onChange={(e) => setQ(e.target.value)} /></div>
          <Segmented value={filter} onChange={setFilter} options={[{ value: "all", label: "All" }, { value: "have", label: "Collected" }, { value: "missing", label: "Missing" }]} />
          <Check label="Include color variations" checked={variants} onChange={setVariants} />
        </div>
        <div className="scroll" style={{ height: H, border: "1px solid var(--border)", borderRadius: 12 }} onScroll={(e) => setScroll((e.target as HTMLDivElement).scrollTop)}>
          <div style={{ height: rows.length * ROW, position: "relative" }}>
            {rows.slice(start, end).map((id, k) => {
              const ri = remakeOf.get(id);
              const vcount = ri !== undefined ? rec.remake.slice(ri * 8, ri * 8 + 8).reduce((a, b) => a + b, 0) : 0;
              return (
                <div key={id} className="list-item" style={{ position: "absolute", top: (start + k) * ROW, left: 0, right: 0, height: ROW, borderRadius: 0, borderBottom: "1px solid var(--border)" }}>
                  <Check checked={rec.items[id] === 1} onChange={(v) => { const [a, b] = setItem(id, v); commit(a, b); }} />
                  <img src={itemSprite(id)} alt="" style={{ width: 24, height: 24, objectFit: "contain" }} loading="lazy" />
                  <span className="ellipsis" style={{ flex: 1 }}>{gd.items[id]}</span>
                  {ri !== undefined && <span className="badge">{vcount} variation{vcount === 1 ? "" : "s"}</span>}
                  <span className="mono tiny faint">{hex(id)}</span>
                </div>
              );
            })}
          </div>
        </div>
      </Card>
    </div></div>
  );
}
