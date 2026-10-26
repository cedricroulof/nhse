import { Sparkles } from "lucide-react";
import React from "react";
import { Button, Card, Field, Loading, NumberInput, Select } from "../components/ui";
import { enumOptions } from "../components/Widgets";
import { api, type Turnips as T } from "../lib/api";
import { useGD, useLoad, useStore } from "../lib/store";

const DAYS = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

export default function Turnips() {
  const { touch } = useStore();
  const gd = useGD();
  const [t, setT] = useLoad(api.turnips, []);
  if (!t) return <Loading />;
  const upd = (next: T) => { setT(next); api.setTurnips(next).then(touch); };
  const max = Math.max(t.buy_price, ...t.sell, 1);
  const H = 180;
  return (
    <div className="content">
      <div className="page">
        <Card title="This week's stalk market" desc="Daisy Mae's Sunday price and Nook's selling prices for each half day"
          actions={<div className="row">
            <Button size="sm" icon={<Sparkles />} onClick={() => upd({ ...t, sell: t.sell.map(() => 660) })}>Set all to 660</Button>
            <Button size="sm" onClick={() => upd({ ...t, sell: t.sell.map((v, i) => (i < 2 ? v : t.buy_price)) })}>Flat (= buy price)</Button>
          </div>}>
          <div style={{ display: "flex", alignItems: "flex-end", gap: 6, height: H + 30, padding: "10px 4px 0" }}>
            <Bar label="Buy" value={t.buy_price} max={max} h={H} color="var(--info)" />
            {t.sell.slice(2).map((v, i) => <Bar key={i} label={`${DAYS[1 + (i >> 1)].slice(0, 3)} ${i % 2 ? "PM" : "AM"}`} value={v} max={max} h={H} color={v >= t.buy_price ? "var(--accent)" : "var(--danger)"} />)}
          </div>
        </Card>
        <div className="grid cols-2">
          <Card title="Prices">
            <div className="grid" style={{ gridTemplateColumns: "110px 1fr 1fr", gap: 10, alignItems: "center" }}>
              <div className="bold">Sunday buy</div>
              <div style={{ gridColumn: "span 2" }}><NumberInput value={t.buy_price} max={9999} suffix="bells" onChange={(v) => upd({ ...t, buy_price: v })} /></div>
              <div /><div className="label">Morning</div><div className="label">Afternoon</div>
              {DAYS.slice(1).map((d, i) => (
                <React.Fragment key={d}>
                  <div className="bold">{d}</div>
                  {[0, 1].map((h) => (
                    <NumberInput key={h} value={t.sell[2 + i * 2 + h]} max={9999}
                      onChange={(v) => { const s = t.sell.slice(); s[2 + i * 2 + h] = v; upd({ ...t, sell: s }); }} />
                  ))}
                </React.Fragment>
              ))}
            </div>
          </Card>
          <Card title="Pattern">
            <div className="col" style={{ gap: 12 }}>
              <Field label="Price pattern" hint="Determines how prices evolve next week"><Select value={t.pattern} onChange={(v) => upd({ ...t, pattern: v })} options={enumOptions(gd, "TurnipPattern")} /></Field>
              <Field label="Spike start (half-day index)"><NumberInput value={t.fever_start} max={14} onChange={(v) => upd({ ...t, fever_start: v })} /></Field>
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
}

function Bar({ label, value, max, h, color }: { label: string; value: number; max: number; h: number; color: string }) {
  return (
    <div className="col" style={{ flex: 1, alignItems: "center", gap: 4, minWidth: 0 }}>
      <div className="tiny bold num">{value}</div>
      <div style={{ width: "100%", maxWidth: 38, height: Math.max(2, (value / max) * h), background: color, borderRadius: "6px 6px 2px 2px", opacity: 0.85, transition: "height .2s" }} />
      <div className="tiny faint nowrap">{label}</div>
    </div>
  );
}
