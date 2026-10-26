import { Plus, Sparkles, X } from "lucide-react";
import { Button, Card, Loading, Select } from "../components/ui";
import { prettify } from "../components/Widgets";
import { api, type Reactions as R } from "../lib/api";
import { useGD, useLoad, useStore } from "../lib/store";

export default function Reactions() {
  const { player, touch, toast } = useStore();
  const gd = useGD();
  const [r, setR] = useLoad(() => api.playerReactions(player), [player]);
  if (!r) return <Loading />;
  const all = (gd.enums.Reaction ?? []).filter(([n, v]) => v !== 0 && !n.startsWith("UNUSED"));
  const name = (v: number) => prettify((gd.enums.Reaction ?? []).find(([, x]) => x === v)?.[0] ?? `#${v}`);
  const upd = (next: R) => { setR(next); api.setPlayerReactions(player, next).then(touch); };
  const known = new Set(r.learned.filter((x) => x !== 0));
  const learnAll = () => {
    const learned = r.learned.slice();
    let n = 0;
    for (const [, v] of all) {
      if (known.has(v)) continue;
      const slot = learned.indexOf(0);
      if (slot < 0) break;
      learned[slot] = v; n++;
    }
    upd({ ...r, learned });
    toast("ok", n ? `Learned ${n} new reactions` : "Every reaction is already known");
  };
  const toggle = (v: number) => {
    const learned = r.learned.slice();
    const i = learned.indexOf(v);
    if (i >= 0) learned[i] = 0;
    else { const s = learned.indexOf(0); if (s < 0) return; learned[s] = v; }
    upd({ ...r, learned });
  };
  return (
    <div className="content">
      <div className="page">
        <Card title="Reaction wheel" desc="The 8 reactions shown on the wheel">
          <div className="grid cols-4" style={{ gap: 10 }}>
            {r.wheel.map((w, i) => (
              <Select key={i} value={w} onChange={(v) => { const wheel = r.wheel.slice(); wheel[i] = v; upd({ ...r, wheel }); }}
                options={[{ value: 0, label: "(empty)" }, ...[...known].map((v) => ({ value: v, label: name(v) }))]} />
            ))}
          </div>
        </Card>
        <Card title="Learned reactions" desc={`${known.size} of ${all.length} known · capacity ${r.learned.length}`}
          actions={<Button size="sm" variant="primary" icon={<Sparkles />} onClick={learnAll}>Learn all</Button>}>
          <div className="chips">
            {all.map(([n, v]) => (
              <button key={v} className={"chip " + (known.has(v) ? "on" : "")} onClick={() => toggle(v)}>
                {known.has(v) ? <X /> : <Plus />}{prettify(n)}
              </button>
            ))}
          </div>
        </Card>
      </div>
    </div>
  );
}
