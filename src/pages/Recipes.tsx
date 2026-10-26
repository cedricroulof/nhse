import { BookCheck, Eraser, Hammer, Search, Sparkles } from "lucide-react";
import { useMemo, useState } from "react";
import { Button, Card, Check, Loading, Segmented } from "../components/ui";
import { api } from "../lib/api";
import { itemSprite } from "../lib/items";
import { useGD, useLoad, useStore } from "../lib/store";

const KITS = new Set([0x1ca, 0x1cd]);
const ROW = 40;

export default function Recipes() {
  const { player, touch, toast } = useStore();
  const gd = useGD();
  const [flags, setFlags] = useLoad(() => api.playerRecipes(player), [player]);
  const [q, setQ] = useState("");
  const [filter, setFilter] = useState<"all" | "known" | "unknown">("all");
  const [scroll, setScroll] = useState(0);
  const rows = useMemo(() => {
    if (!flags) return [];
    const s = q.trim().toLowerCase();
    return gd.recipes
      .map(([idx, item]) => ({ idx, item, name: gd.items[item] ?? `#${item}` }))
      .filter((r) => (!s || r.name.toLowerCase().includes(s) || String(r.idx) === s) && (filter === "all" || ((flags[r.idx] & 1) !== 0) === (filter === "known")))
      .sort((a, b) => a.name.localeCompare(b.name));
  }, [gd, flags, q, filter]);
  if (!flags) return <Loading />;
  const commit = (next: number[]) => { setFlags(next); api.setPlayerRecipes(player, next).then(touch); };
  const bit = (i: number, b: number, v: boolean) => { const n = flags.slice(); n[i] = v ? n[i] | (1 << b) : n[i] & ~(1 << b); commit(n); };
  const known = gd.recipes.filter(([i]) => flags[i] & 1).length;
  const H = 600;
  const start = Math.max(0, Math.floor(scroll / ROW) - 6);
  const end = Math.min(rows.length, start + Math.ceil(H / ROW) + 12);
  return (
    <div className="content"><div className="page">
      <Card title={<span className="row"><Hammer size={16} /> DIY recipes</span>} desc={`${known} of ${gd.recipes.length} learned`}
        actions={<div className="row">
          <Button size="sm" variant="primary" icon={<Sparkles />} onClick={() => {
            const n = flags.slice(); let c = 0;
            for (const [i] of gd.recipes) if (!KITS.has(i) && !(n[i] & 1)) { n[i] |= 1 | 4; c++; }
            commit(n); toast("ok", `Learned ${c} recipes`, "Construction kits for bridges and the campsite are skipped — they unlock through the story.");
          }}>Learn all</Button>
          <Button size="sm" icon={<BookCheck />} onClick={() => commit(flags.map((f) => (f & 1 ? (f | 2) & ~4 : f)))}>Mark known as crafted</Button>
          <Button size="sm" variant="danger" icon={<Eraser />} onClick={() => commit(flags.map(() => 0))}>Forget all</Button>
        </div>}>
        <div className="row" style={{ marginBottom: 10 }}>
          <div className="input-wrap" style={{ width: 280 }}><Search /><input className="input" placeholder="Search recipes…" value={q} onChange={(e) => setQ(e.target.value)} /></div>
          <Segmented value={filter} onChange={setFilter} options={[{ value: "all", label: "All" }, { value: "known", label: "Learned" }, { value: "unknown", label: "Not learned" }]} />
        </div>
        <div className="row tiny faint" style={{ padding: "0 10px 6px", gap: 0 }}>
          <span style={{ flex: 1 }}>Recipe</span>{["Learned", "Crafted", "New", "Favorite"].map((h) => <span key={h} style={{ width: 80, textAlign: "center" }}>{h}</span>)}
        </div>
        <div className="scroll" style={{ height: H, border: "1px solid var(--border)", borderRadius: 12 }} onScroll={(e) => setScroll((e.target as HTMLDivElement).scrollTop)}>
          <div style={{ height: rows.length * ROW, position: "relative" }}>
            {rows.slice(start, end).map((r, k) => (
              <div key={r.idx} className="list-item" style={{ position: "absolute", top: (start + k) * ROW, left: 0, right: 0, height: ROW, borderRadius: 0, borderBottom: "1px solid var(--border)" }}>
                <img src={itemSprite(r.item)} alt="" style={{ width: 26, height: 26, objectFit: "contain" }} loading="lazy" />
                <span className="ellipsis" style={{ flex: 1 }}>{r.name}{KITS.has(r.idx) && <span className="badge warn" style={{ marginLeft: 6 }}>story</span>}</span>
                {[0, 1, 2, 3].map((b) => <span key={b} style={{ width: 80, display: "grid", placeItems: "center" }}><Check checked={(flags[r.idx] & (1 << b)) !== 0} onChange={(v) => bit(r.idx, b, v)} /></span>)}
              </div>
            ))}
          </div>
        </div>
      </Card>
    </div></div>
  );
}
