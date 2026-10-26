import { RotateCcw, Search } from "lucide-react";
import { useMemo, useState } from "react";
import type { FlagInfo } from "../lib/api";
import { Button, Check, NumberInput } from "./ui";

const ROW = 38;

/** Editable table of event flags (virtualized). */
export function FlagTable({ values, defs, onChange, height = 560 }: { values: number[]; defs: FlagInfo[]; onChange: (v: number[]) => void; height?: number }) {
  const [q, setQ] = useState("");
  const [changedOnly, setChangedOnly] = useState(false);
  const [showUnknown, setShowUnknown] = useState(false);
  const [scroll, setScroll] = useState(0);
  const byIndex = useMemo(() => new Map(defs.map((d) => [d.index, d])), [defs]);
  const rows = useMemo(() => {
    const idx = showUnknown ? values.map((_, i) => i) : defs.map((d) => d.index).filter((i) => i < values.length);
    const s = q.trim().toLowerCase();
    return idx.filter((i) => {
      const d = byIndex.get(i);
      if (changedOnly && (!d || values[i] === d.default)) return false;
      if (!s) return true;
      return String(i) === s || (d && (d.label + " " + d.name + " " + d.desc).toLowerCase().includes(s));
    });
  }, [values, defs, q, changedOnly, showUnknown, byIndex]);
  const start = Math.max(0, Math.floor(scroll / ROW) - 5);
  const end = Math.min(rows.length, start + Math.ceil(height / ROW) + 10);
  const set = (i: number, v: number) => { const n = values.slice(); n[i] = v; onChange(n); };
  return (
    <div className="col" style={{ gap: 10 }}>
      <div className="row wrap">
        <div className="input-wrap" style={{ width: 280 }}><Search /><input className="input" placeholder="Search flags…" value={q} onChange={(e) => setQ(e.target.value)} /></div>
        <Check label="Changed from default" checked={changedOnly} onChange={setChangedOnly} />
        <Check label="Show unnamed slots" checked={showUnknown} onChange={setShowUnknown} />
        <span className="spacer" />
        <span className="small faint">{rows.length} flags</span>
      </div>
      <div className="scroll" style={{ height, border: "1px solid var(--border)", borderRadius: 12 }} onScroll={(e) => setScroll((e.target as HTMLDivElement).scrollTop)}>
        <div style={{ height: rows.length * ROW, position: "relative" }}>
          {rows.slice(start, end).map((i, k) => {
            const d = byIndex.get(i);
            const v = values[i];
            const boolish = d && d.max === 1;
            return (
              <div key={i} className="list-item" style={{ position: "absolute", top: (start + k) * ROW, left: 0, right: 0, height: ROW, borderRadius: 0, borderBottom: "1px solid var(--border)" }}>
                <span className="mono tiny faint" style={{ width: 44 }}>{i}</span>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div className="ellipsis" style={{ fontWeight: 550 }}>{d ? d.label : "—"}</div>
                  {d?.desc && <div className="tiny faint ellipsis">{d.desc}</div>}
                </div>
                {boolish ? <Check checked={v !== 0} onChange={(x) => set(i, x ? 1 : 0)} />
                  : <NumberInput value={v} min={-32768} max={32767} width={110} onChange={(x) => set(i, x)} />}
                <span style={{ width: 30 }}>{d && v !== d.default && <Button size="sm" variant="ghost" iconOnly icon={<RotateCcw />} title={`Reset to ${d.default}`} onClick={() => set(i, d.default)} />}</span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
