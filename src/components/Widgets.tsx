import { ChevronDown, Search } from "lucide-react";
import React, { useMemo, useRef, useState } from "react";
import type { GDate, GameData } from "../lib/api";
import { villagerSprite } from "../lib/items";
import { useStore } from "../lib/store";
import { PickerPopover } from "./ItemPicker";

/** Searchable villager chooser with portraits. `value` is an internal name like `cat00`. */
export function VillagerPicker({ value, onChange, onlyTemplates, width }: { value: string; onChange: (internal: string) => void; onlyTemplates?: boolean; width?: number | string }) {
  const { gd } = useStore();
  const [open, setOpen] = useState(false);
  const [q, setQ] = useState("");
  const ref = useRef<HTMLButtonElement>(null);
  const list = useMemo(() => {
    const all = (gd?.villagers ?? []).filter((v) => !onlyTemplates || v.has_template);
    const s = q.trim().toLowerCase();
    return s ? all.filter((v) => v.name.toLowerCase().includes(s) || v.internal.includes(s)) : all;
  }, [gd, q, onlyTemplates]);
  const cur = gd?.villagers.find((v) => v.internal === value);
  return (
    <>
      <button ref={ref} className="input" style={{ display: "flex", alignItems: "center", gap: 8, width, cursor: "default" }} onClick={() => setOpen(true)}>
        <img src={villagerSprite(value)} alt="" style={{ width: 20, height: 20, objectFit: "contain" }} />
        <span className="ellipsis" style={{ flex: 1, textAlign: "left" }}>{cur ? cur.name : value}</span>
        <span className="tiny faint mono">{value}</span>
        <ChevronDown size={14} style={{ color: "var(--text-3)" }} />
      </button>
      {open && (
        <PickerPopover anchor={ref.current} onClose={() => setOpen(false)} width={460}>
          <div className="col">
            <div className="input-wrap"><Search /><input className="input" autoFocus placeholder="Search villagers…" value={q} onChange={(e) => setQ(e.target.value)} /></div>
            <div className="scroll" style={{ maxHeight: 360, display: "grid", gridTemplateColumns: "repeat(5, 1fr)", gap: 6 }}>
              {list.map((v) => (
                <button key={v.internal} className={"vcard " + (v.internal === value ? "sel" : "")} style={{ padding: "8px 4px" }}
                  onClick={() => { onChange(v.internal); setOpen(false); }}>
                  <img src={villagerSprite(v.internal)} alt="" className="portrait sm" loading="lazy" />
                  <div className="n tiny ellipsis" style={{ maxWidth: "100%" }}>{v.name}</div>
                </button>
              ))}
            </div>
          </div>
        </PickerPopover>
      )}
    </>
  );
}

const pad = (n: number) => String(n).padStart(2, "0");
export function DateInput({ value, onChange, allowEmpty }: { value: GDate; onChange: (d: GDate) => void; allowEmpty?: boolean }) {
  const empty = value.year === 0 && value.month === 0 && value.day === 0;
  const valid = value.year >= 1 && value.month >= 1 && value.month <= 12 && value.day >= 1 && value.day <= 31;
  return (
    <div className="row" style={{ gap: 6 }}>
      <input type="date" className="input" style={{ width: 150 }} value={valid ? `${String(value.year).padStart(4, "0")}-${pad(value.month)}-${pad(value.day)}` : ""}
        onChange={(e) => {
          const [y, m, d] = e.target.value.split("-").map(Number);
          if (y && m && d) onChange({ year: y, month: m, day: d });
          else if (allowEmpty) onChange({ year: 0, month: 0, day: 0 });
        }} />
      {allowEmpty && !empty && <button className="btn sm ghost" onClick={() => onChange({ year: 0, month: 0, day: 0 })}>Clear</button>}
    </div>
  );
}

export function enumOptions(gd: GameData, name: string, pretty = true) {
  return (gd.enums[name] ?? []).map(([label, value]) => ({ value, label: pretty ? prettify(label) : label }));
}

export const prettify = (s: string) => s.replace(/_/g, " ").replace(/([a-z])([A-Z])/g, "$1 $2").replace(/^Kind /, "");

export function fruitOptions(gd: GameData) {
  return (gd.lists.Fruits ?? []).map((id) => ({ value: id, label: gd.items[id] }));
}

export function MonthDay({ month, day, onChange }: { month: number; day: number; onChange: (m: number, d: number) => void }) {
  const months = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
  const days = new Date(2020, month, 0).getDate() || 31;
  return (
    <div className="row" style={{ gap: 6 }}>
      <select className="select" style={{ width: 130 }} value={month} onChange={(e) => onChange(Number(e.target.value), Math.min(day, new Date(2020, Number(e.target.value), 0).getDate()))}>
        {months.map((m, i) => <option key={i} value={i + 1}>{m}</option>)}
      </select>
      <select className="select" style={{ width: 70 }} value={day} onChange={(e) => onChange(month, Number(e.target.value))}>
        {Array.from({ length: days }, (_, i) => <option key={i} value={i + 1}>{i + 1}</option>)}
      </select>
    </div>
  );
}

export function PageHeader({ children }: { children: React.ReactNode }) {
  return <div className="row" style={{ gap: 10, flexWrap: "wrap" }}>{children}</div>;
}
