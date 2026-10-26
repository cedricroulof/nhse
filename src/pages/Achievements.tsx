import { Award, Search, Sparkles } from "lucide-react";
import { useMemo, useState } from "react";
import { Button, Card, Loading, NumberInput } from "../components/ui";
import { prettify } from "../components/Widgets";
import { api, type Achievements as A } from "../lib/api";
import { fmtDate } from "../lib/items";
import { useGD, useLoad, useStore } from "../lib/store";

export default function Achievements() {
  const { player, touch, toast, confirm } = useStore();
  const gd = useGD();
  const [a, setA] = useLoad(() => api.playerAchievements(player), [player]);
  const [q, setQ] = useState("");
  const list = useMemo(() => gd.achievements.filter((d) => !q || (d.label + d.name).toLowerCase().includes(q.toLowerCase())), [gd, q]);
  if (!a) return <Loading />;
  const upd = (next: A) => { setA(next); api.setPlayerAchievements(player, next).then(touch); };
  const done = gd.achievements.filter((d) => a.read[d.index]?.slice(0, d.tiers).every(Boolean)).length;
  return (
    <div className="content">
      <div className="page">
        <Card title={<span className="row"><Award size={16} /> Nook Miles achievements</span>} desc={`${done} of ${gd.achievements.length} fully completed`}
          actions={<div className="row">
            <div className="input-wrap" style={{ width: 220 }}><Search /><input className="input" placeholder="Filter…" value={q} onChange={(e) => setQ(e.target.value)} /></div>
            <Button size="sm" variant="primary" icon={<Sparkles />} onClick={async () => {
              if (!(await confirm({ title: "Complete every achievement?", message: "Counters are raised to the final tier and every milestone is marked as earned (dates start today)." }))) return;
              setA(await api.completeAchievements(player)); touch(); toast("ok", "All achievements completed");
            }}>Complete all</Button>
          </div>}>
          <table className="table">
            <thead><tr><th>Achievement</th><th style={{ width: 150 }}>Count</th><th>Milestones</th><th style={{ width: 90 }}></th></tr></thead>
            <tbody>
              {list.map((d) => {
                const i = d.index;
                return (
                  <tr key={i}>
                    <td><div className="bold">{prettify(d.label)}</div><div className="tiny faint">{d.thresholds.filter(Boolean).join(" · ")}</div></td>
                    <td><NumberInput value={a.counts[i]} onChange={(v) => { const c = a.counts.slice(); c[i] = v; upd({ ...a, counts: c }); }} /></td>
                    <td>
                      <div className="row" style={{ gap: 4, flexWrap: "wrap" }}>
                        {Array.from({ length: d.tiers }, (_, t) => (
                          <span key={t} className={"badge " + (a.read[i][t] ? "accent" : "")} title={fmtDate(a.dates[i][t])}>{t + 1}{a.read[i][t] ? " ✓" : ""}</span>
                        ))}
                      </div>
                    </td>
                    <td><Button size="sm" onClick={async () => { setA(await api.completeAchievements(player, [i])); touch(); }}>Complete</Button></td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </Card>
      </div>
    </div>
  );
}
