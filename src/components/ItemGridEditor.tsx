import { ClipboardPaste, Copy, Download, Eraser, ListPlus, PaintBucket, Trash2, Upload } from "lucide-react";
import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api, NONE, type Item } from "../lib/api";
import { F, pickFile, saveFile } from "../lib/dialog";
import { itemLabel } from "../lib/items";
import { useStore } from "../lib/store";
import { ItemEditor } from "./ItemEditor";
import { ItemSlot } from "./ItemSlot";
import { Button, Modal, Popover, type MenuEntry } from "./ui";

let clipboard: Item[] = [];

/** Grid of item slots with multi-select, drag & drop, copy/paste and a side editor. */
export function ItemGridEditor({ items, onChange, columns = 10, unlocked, title, pageSize, allowWrap = true, toolbar, slotSize, fileName = "items" }: {
  items: Item[]; onChange: (items: Item[]) => void; columns?: number; unlocked?: number; title?: React.ReactNode; pageSize?: number;
  allowWrap?: boolean; toolbar?: React.ReactNode; slotSize?: number; fileName?: string;
}) {
  const { gd, toast, confirm } = useStore();
  const [sel, setSel] = useState<number[]>([0]);
  const [anchor, setAnchor] = useState(0);
  const [page, setPage] = useState(0);
  const [menu, setMenu] = useState<{ x: number; y: number } | null>(null);
  const [dragFrom, setDragFrom] = useState<number | null>(null);
  const [dropAt, setDropAt] = useState<number | null>(null);
  const [pasteOpen, setPasteOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const per = pageSize ?? items.length;
  const pages = Math.max(1, Math.ceil(items.length / per));
  const first = page * per;
  const visible = items.slice(first, first + per);
  const current = sel.length ? sel[sel.length - 1] : 0;
  const selSet = useMemo(() => new Set(sel), [sel]);

  useEffect(() => { if (current >= items.length) setSel([0]); }, [items.length, current]);

  const update = useCallback((idx: number[], f: (it: Item, i: number) => Item) => {
    const next = items.slice();
    for (const i of idx) next[i] = f(next[i], i);
    onChange(next);
  }, [items, onChange]);

  const click = (i: number, e: React.MouseEvent) => {
    if (e.button === 2) {
      if (!selSet.has(i)) { setSel([i]); setAnchor(i); }
      return;
    }
    if (e.shiftKey) {
      const [a, b] = [Math.min(anchor, i), Math.max(anchor, i)];
      setSel(Array.from({ length: b - a + 1 }, (_, k) => a + k));
    } else if (e.metaKey || e.ctrlKey) {
      setSel(selSet.has(i) ? sel.filter((x) => x !== i) : [...sel, i]);
      setAnchor(i);
    } else {
      setSel([i]);
      setAnchor(i);
    }
  };

  const copy = () => { clipboard = sel.slice().sort((a, b) => a - b).map((i) => items[i]); toast("info", `Copied ${clipboard.length} item${clipboard.length > 1 ? "s" : ""}`); };
  const paste = () => {
    if (!clipboard.length) return;
    const start = Math.min(...sel);
    if (sel.length > 1 && clipboard.length === 1) update(sel, () => clipboard[0]);
    else {
      const next = items.slice();
      clipboard.forEach((it, k) => { if (start + k < next.length) next[start + k] = it; });
      onChange(next);
    }
  };
  const del = () => update(sel, () => NONE);

  useEffect(() => {
    const el = rootRef.current;
    if (!el) return;
    const onKey = (e: KeyboardEvent) => {
      if (!el.contains(document.activeElement) && document.activeElement !== el) return;
      const tag = (document.activeElement as HTMLElement)?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;
      const mod = e.metaKey || e.ctrlKey;
      if (mod && e.key === "c") { e.preventDefault(); copy(); }
      else if (mod && e.key === "v") { e.preventDefault(); paste(); }
      else if (mod && e.key === "a") { e.preventDefault(); setSel(items.map((_, i) => i)); }
      else if (e.key === "Delete" || e.key === "Backspace") { e.preventDefault(); del(); }
      else if (["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(e.key)) {
        e.preventDefault();
        const d = e.key === "ArrowLeft" ? -1 : e.key === "ArrowRight" ? 1 : e.key === "ArrowUp" ? -columns : columns;
        const n = Math.max(0, Math.min(items.length - 1, current + d));
        setSel([n]); setAnchor(n);
        if (n < first || n >= first + per) setPage(Math.floor(n / per));
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  const menuItems: MenuEntry[] = [
    { label: "Copy", icon: <Copy />, onClick: copy },
    { label: "Paste", icon: <ClipboardPaste />, onClick: paste, disabled: !clipboard.length },
    { label: sel.length > 1 ? `Apply current item to ${sel.length} slots` : "Duplicate to all empty slots", icon: <PaintBucket />, onClick: () => {
      const src = items[current];
      if (sel.length > 1) update(sel, () => src);
      else update(items.map((_, i) => i).filter((i) => items[i].id === 0xfffe), () => src);
    } },
    "sep",
    { label: sel.length > 1 ? `Clear ${sel.length} slots` : "Clear slot", icon: <Eraser />, onClick: del, danger: true },
  ];

  const exportFile = async () => {
    const p = await saveFile("Export items", `${fileName}.nhi`, [F.nhi]);
    if (p) { await api.exportItemsFile(items, p); toast("ok", "Exported", p); }
  };
  const importFile = async () => {
    const p = await pickFile("Import items", [F.nhi, F.all]);
    if (!p) return;
    try {
      const imported = await api.importItemsFile(p);
      const next = items.slice();
      let k = 0;
      for (let i = 0; i < next.length && k < imported.length; i++) {
        if (next[i].id !== 0xfffe) continue;
        next[i] = imported[k++];
      }
      onChange(next);
      toast("ok", `Imported ${k} items into empty slots`, imported.length > k ? `${imported.length - k} did not fit.` : undefined);
    } catch (e) { toast("error", "Import failed", String(e)); }
  };

  const filled = items.filter((i) => i.id !== 0xfffe).length;

  return (
    <div className="row" style={{ alignItems: "flex-start", gap: 16 }}>
      <div ref={rootRef} tabIndex={0} className="card" style={{ flex: "none", outline: "none" }}>
        <div className="card-head" style={{ paddingBottom: 4 }}>
          <h2>{title}<div className="desc">{filled} of {items.length} slots used{unlocked !== undefined && unlocked < items.length ? ` · ${unlocked} unlocked in game` : ""}</div></h2>
          {toolbar}
          <Button size="sm" icon={<ListPlus />} onClick={() => setPasteOpen(true)} title="Paste a list of item names or hex codes">Add list</Button>
          <Button size="sm" iconOnly icon={<Upload />} onClick={importFile} title="Import .nhi" />
          <Button size="sm" iconOnly icon={<Download />} onClick={exportFile} title="Export .nhi" />
          <Button size="sm" iconOnly variant="danger" icon={<Trash2 />} title="Clear all" onClick={async () => {
            if (await confirm({ title: "Clear every slot?", message: "All items in this list will be removed.", danger: true, confirm: "Clear all" })) onChange(items.map(() => NONE));
          }} />
        </div>
        <div className="card-body" style={{ paddingTop: 8 }}>
          <div className="slot-grid" style={{ gridTemplateColumns: `repeat(${columns}, ${slotSize ?? 56}px)` }}>
            {visible.map((it, k) => {
              const i = first + k;
              return (
                <ItemSlot key={i} item={it} size={slotSize} selected={i === current} multi={selSet.has(i) && sel.length > 1}
                  locked={unlocked !== undefined && i >= unlocked} drop={dropAt === i}
                  onMouseDown={(e) => { rootRef.current?.focus(); click(i, e); }}
                  onContextMenu={(e) => { e.preventDefault(); setMenu({ x: e.clientX, y: e.clientY }); }}
                  draggable
                  onDragStart={(e) => { setDragFrom(i); e.dataTransfer.effectAllowed = "move"; }}
                  onDragOver={(e) => { e.preventDefault(); setDropAt(i); }}
                  onDrop={(e) => {
                    e.preventDefault(); setDropAt(null);
                    if (dragFrom === null || dragFrom === i) return;
                    const next = items.slice();
                    [next[i], next[dragFrom]] = [next[dragFrom], next[i]];
                    onChange(next); setSel([i]); setDragFrom(null);
                  }}
                />
              );
            })}
          </div>
          {pages > 1 && (
            <div className="row" style={{ marginTop: 12, justifyContent: "center", flexWrap: "wrap", gap: 4 }}>
              {Array.from({ length: pages }, (_, p) => (
                <button key={p} className={"chip " + (p === page ? "on" : "")} onClick={() => setPage(p)} style={{ height: 24 }}>{p + 1}</button>
              ))}
            </div>
          )}
          <div className="tiny faint" style={{ marginTop: 10 }}>Click to select · ⇧ range · ⌘ toggle · drag to swap · ⌘C / ⌘V · ⌫ clears</div>
        </div>
      </div>
      <div className="card" style={{ flex: 1, minWidth: 300, position: "sticky", top: 0 }}>
        <div className="card-head"><h2>{sel.length > 1 ? `${sel.length} slots selected` : `Slot ${current + 1}`}
          <div className="desc">{gd && items[current] ? itemLabel(gd, items[current]) : ""}</div></h2></div>
        <div className="card-body">
          {items[current] && (
            <ItemEditor item={items[current]} allowWrap={allowWrap}
              onChange={(it) => update(sel.length > 1 ? sel : [current], () => it)} />
          )}
          {sel.length > 1 && <div className="tiny faint" style={{ marginTop: 10 }}>Changes apply to every selected slot.</div>}
        </div>
      </div>
      {menu && <Popover at={menu} items={menuItems} onClose={() => setMenu(null)} />}
      {pasteOpen && <PasteListModal onClose={() => setPasteOpen(false)} onAdd={(list) => {
        const next = items.slice();
        let k = 0;
        for (let i = 0; i < next.length && k < list.length; i++) if (next[i].id === 0xfffe) next[i] = list[k++];
        onChange(next);
        toast("ok", `Added ${k} item${k === 1 ? "" : "s"}`, list.length > k ? `${list.length - k} did not fit (no empty slots).` : undefined);
      }} />}
    </div>
  );
}

export function PasteListModal({ onClose, onAdd }: { onClose: () => void; onAdd: (items: Item[]) => void }) {
  const [text, setText] = useState("");
  const [errors, setErrors] = useState<string[]>([]);
  return (
    <Modal title="Add items from a list" onClose={onClose} footer={<>
      <Button onClick={onClose}>Cancel</Button>
      <Button variant="primary" onClick={async () => {
        const [items, errs] = await api.parseItems(text);
        setErrors(errs);
        if (items.length) { onAdd(items); if (!errs.length) onClose(); }
      }}>Add items</Button>
    </>}>
      <div className="col">
        <div className="small muted">One per line (or comma separated): item names in the current language, or hex codes like <code>0A54</code> / <code>0000000900000A54</code>.</div>
        <textarea className="textarea mono" rows={10} value={text} onChange={(e) => setText(e.target.value)} placeholder={"wooden chair\nnet\n0x0C0A"} />
        {errors.length > 0 && <div className="callout warn small">Not recognised: {errors.join(", ")}</div>}
      </div>
    </Modal>
  );
}
