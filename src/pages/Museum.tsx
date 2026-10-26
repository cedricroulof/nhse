import { ArrowDownWideNarrow, Bug, Fish, Gem, Landmark, Shell, Sparkles, Trash2 } from "lucide-react";
import { useMemo, useState } from "react";
import { Button, Card, Check, Field, Loading, NumberInput, Select } from "../components/ui";
import { api, type Museum as M } from "../lib/api";
import { fmtDate, itemLabel, itemSprite } from "../lib/items";
import { useGD, useLoad, useStore } from "../lib/store";

const CATS: [string, string, React.ReactNode][] = [["fish", "Fish", <Fish />], ["bugs", "Bugs", <Bug />], ["sea", "Sea creatures", <Shell />], ["fossils", "Fossils", <Gem />], ["art", "Art (genuine)", <Landmark />]];

export default function Museum() {
  const { touch, toast, save } = useStore();
  const gd = useGD();
  const [m, setM] = useLoad(api.museum, []);
  const [cats, setCats] = useState<string[]>(["fish", "bugs", "sea", "fossils", "art"]);
  const [who, setWho] = useState(0);
  const [perDay, setPerDay] = useState(20);
  const entries = useMemo(() => (m ? m.items.map((it, i) => ({ it, i })).filter((e) => e.it.id !== 0xfffe) : []), [m]);
  if (!m) return <Loading />;
  const commit = (next: M) => { setM(next); api.setMuseum(next).then(touch); };
  const counts = CATS.map(([k]) => {
    const list = new Set(gd.lists[k === "bugs" ? "Bugs" : k === "fish" ? "Fish" : k === "sea" ? "Dive" : k === "fossils" ? "Fossils" : "Art"] ?? []);
    return [entries.filter((e) => list.has(e.it.id)).length, list.size];
  });
  return (
    <div className="content"><div className="page">
      <div className="grid cols-4" style={{ gridTemplateColumns: "repeat(5, 1fr)" }}>
        {CATS.map(([k, l, icon], i) => (
          <div key={k} className="card stat"><div className="k">{icon}{l}</div><div className="v">{counts[i][0]}<span className="faint" style={{ fontSize: 14 }}> / {counts[i][1]}</span></div></div>
        ))}
      </div>
      <Card title="Donate everything" desc="Adds every missing specimen, a few per day, credited to a resident">
        <div className="row wrap" style={{ gap: 14, alignItems: "flex-end" }}>
          {CATS.map(([k, l]) => <Check key={k} label={l} checked={cats.includes(k)} onChange={(v) => setCats(v ? [...cats, k] : cats.filter((c) => c !== k))} />)}
          <Field label="Donated by"><Select value={who} onChange={setWho} options={(save?.players ?? []).map((p) => ({ value: p.index, label: p.name }))} style={{ width: 150 }} /></Field>
          <Field label="Per day"><NumberInput value={perDay} min={1} max={500} onChange={setPerDay} width={80} /></Field>
          <Button variant="primary" icon={<Sparkles />} onClick={async () => { const [next, n] = await api.museumDonateAll(cats, who, perDay); setM(next); touch(); toast("ok", n ? `Donated ${n} items` : "Nothing left to donate"); }}>Donate</Button>
        </div>
      </Card>
      <Card title={`Donations (${entries.length})`} actions={<Button size="sm" icon={<ArrowDownWideNarrow />} onClick={async () => { setM(await api.museumSort()); touch(); }}>Sort by date</Button>}>
        <table className="table">
          <thead><tr><th>Item</th><th>Date</th><th>Donated by</th><th style={{ width: 50 }}></th></tr></thead>
          <tbody>
            {entries.map(({ it, i }) => (
              <tr key={i}>
                <td><div className="row"><img src={itemSprite(it.id)} style={{ width: 26, height: 26, objectFit: "contain" }} alt="" loading="lazy" />{itemLabel(gd, it)}</div></td>
                <td>{fmtDate(m.dates[i])}</td>
                <td>{save?.players[m.players[i]]?.name ?? `Player ${m.players[i] + 1}`}</td>
                <td><Button size="sm" variant="ghost" iconOnly icon={<Trash2 />} onClick={() => {
                  const next = { ...m, items: m.items.slice(), dates: m.dates.slice(), players: m.players.slice() };
                  next.items[i] = { id: 0xfffe, system: 0, additional: 0, free: 0 }; next.dates[i] = { year: 0, month: 0, day: 0 }; next.players[i] = 0;
                  commit(next);
                }} /></td>
              </tr>
            ))}
          </tbody>
        </table>
      </Card>
    </div></div>
  );
}
