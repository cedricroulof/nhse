import { ChevronDown, Search } from "lucide-react";
import React, { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { baseName, hex, itemSprite, kindName, NONE_ID, searchItems, type SearchEntry } from "../lib/items";
import { useStore } from "../lib/store";

const ROW = 34;

/** Virtualized, searchable item list. Supports words, `*`/`?` wildcards, /regex/ and ids. */
export function ItemSearchList({ value, onPick, filter, autoFocus = true, height = 380, entries }: {
  value?: number; onPick: (id: number) => void; filter?: (e: SearchEntry) => boolean; autoFocus?: boolean; height?: number; entries?: SearchEntry[];
}) {
  const { index, indexAll, gd, settings } = useStore();
  const [q, setQ] = useState("");
  const [showAll, setShowAll] = useState(false);
  const base = entries ?? (showAll ? indexAll : index);
  const results = useMemo(() => {
    const r = searchItems(base, q, 20000);
    return filter ? r.filter(filter) : r;
  }, [base, q, filter]);
  const [hi, setHi] = useState(0);
  const [scroll, setScroll] = useState(0);
  const listRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const i = value === undefined ? 0 : results.findIndex((r) => r.id === value);
    setHi(Math.max(0, i));
    if (i > 0 && listRef.current) listRef.current.scrollTop = Math.max(0, i * ROW - height / 2);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [results]);
  const start = Math.max(0, Math.floor(scroll / ROW) - 6);
  const end = Math.min(results.length, start + Math.ceil(height / ROW) + 12);
  const ensure = (i: number) => {
    const el = listRef.current;
    if (!el) return;
    if (i * ROW < el.scrollTop) el.scrollTop = i * ROW;
    else if ((i + 1) * ROW > el.scrollTop + height) el.scrollTop = (i + 1) * ROW - height;
  };
  return (
    <div className="col" style={{ gap: 8 }}>
      <div className="input-wrap">
        <Search />
        <input
          className="input"
          autoFocus={autoFocus}
          placeholder="Search items…  (try  * painting,  /^wooden/,  0x0A54)"
          value={q}
          spellCheck={false}
          onChange={(e) => setQ(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "ArrowDown") { e.preventDefault(); const n = Math.min(results.length - 1, hi + 1); setHi(n); ensure(n); }
            if (e.key === "ArrowUp") { e.preventDefault(); const n = Math.max(0, hi - 1); setHi(n); ensure(n); }
            if (e.key === "Enter" && results[hi]) onPick(results[hi].id);
          }}
        />
      </div>
      <div className="row small faint" style={{ justifyContent: "space-between" }}>
        <span>{results.length.toLocaleString()} result{results.length === 1 ? "" : "s"}</span>
        {!entries && (
          <label className="check tiny">
            <input type="checkbox" checked={showAll} onChange={(e) => setShowAll(e.target.checked)} /> include unnamed / internal items
          </label>
        )}
      </div>
      <div ref={listRef} className="scroll" style={{ height, position: "relative", borderRadius: 10, border: "1px solid var(--border)" }}
        onScroll={(e) => setScroll((e.target as HTMLDivElement).scrollTop)}>
        <div style={{ height: results.length * ROW, position: "relative" }}>
          {results.slice(start, end).map((r, k) => {
            const i = start + k;
            return (
              <div key={r.id + ":" + i} className={"list-item " + (i === hi ? "sel" : "")}
                style={{ position: "absolute", top: i * ROW, left: 4, right: 4, height: ROW - 2, padding: "0 8px" }}
                onMouseEnter={() => setHi(i)} onMouseDown={() => onPick(r.id)}>
                {r.id !== NONE_ID ? <img src={itemSprite(r.id)} alt="" style={{ width: 24, height: 24, objectFit: "contain" }} loading="lazy" /> : <span style={{ width: 24 }} />}
                <span className="ellipsis" style={{ flex: 1 }}>{r.name}</span>
                {gd && settings?.advanced && <span className="tiny faint">{kindName(gd, r.id).replace(/^Kind_/, "")}</span>}
                <span className="tiny faint mono">{r.id === NONE_ID ? "" : hex(r.id)}</span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

/** Button that shows the selected item and opens a search popover. */
export function ItemPicker({ value, onChange, filter, placeholder = "Choose an item", width, entries, disabled }: {
  value: number; onChange: (id: number) => void; filter?: (e: SearchEntry) => boolean; placeholder?: string; width?: number | string;
  entries?: SearchEntry[]; disabled?: boolean;
}) {
  const { gd } = useStore();
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLButtonElement>(null);
  const name = gd ? baseName(gd, value) : "";
  return (
    <>
      <button ref={ref} className="input" disabled={disabled} style={{ display: "flex", alignItems: "center", gap: 8, textAlign: "left", width, cursor: "default" }} onClick={() => setOpen(true)}>
        {value !== NONE_ID && <img src={itemSprite(value)} alt="" style={{ width: 20, height: 20, objectFit: "contain" }} />}
        <span className="ellipsis" style={{ flex: 1, color: value === NONE_ID ? "var(--text-3)" : undefined }}>{value === NONE_ID ? placeholder : name}</span>
        <ChevronDown size={14} style={{ color: "var(--text-3)", flex: "none" }} />
      </button>
      {open && <PickerPopover anchor={ref.current} onClose={() => setOpen(false)}>
        <ItemSearchList value={value} filter={filter} entries={entries} onPick={(id) => { onChange(id); setOpen(false); }} />
      </PickerPopover>}
    </>
  );
}

export function PickerPopover({ anchor, onClose, children, width = 420 }: { anchor: HTMLElement | null; onClose: () => void; children: React.ReactNode; width?: number }) {
  const ref = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState({ x: -9999, y: -9999 });
  useLayoutEffect(() => {
    if (!anchor || !ref.current) return;
    const r = anchor.getBoundingClientRect();
    const h = ref.current.offsetHeight;
    let x = Math.min(r.left, window.innerWidth - width - 10);
    let y = r.bottom + 4;
    if (y + h > window.innerHeight - 10) y = Math.max(10, r.top - h - 4);
    x = Math.max(10, x);
    setPos({ x, y });
  }, [anchor, width]);
  useEffect(() => {
    const d = (e: MouseEvent) => { if (ref.current && !ref.current.contains(e.target as Node)) onClose(); };
    const k = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
    setTimeout(() => window.addEventListener("mousedown", d), 0);
    window.addEventListener("keydown", k);
    return () => { window.removeEventListener("mousedown", d); window.removeEventListener("keydown", k); };
  }, [onClose]);
  return createPortal(
    <div ref={ref} className="popover" style={{ left: pos.x, top: pos.y, width, padding: 10 }}>{children}</div>,
    document.body,
  );
}
