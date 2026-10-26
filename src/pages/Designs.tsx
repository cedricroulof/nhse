import { Download, Droplet, Eraser, FileDown, FileUp, ImageDown, ImageUp, PaintBucket, Palette, Pencil, Pipette, UserCheck } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Button, Card, Field, Loading, MenuButton, Modal, Segmented, Tabs, TextInput } from "../components/ui";
import { api, type DesignKind, type DesignPixels } from "../lib/api";
import { F, pickFile, saveFile } from "../lib/dialog";
import { asset } from "../lib/items";
import { useLoad, useStore } from "../lib/store";

const SHEETS = ["Front", "Back", "Right", "Left"];

export default function Designs() {
  const { touch, toast, rev, player, save, confirm } = useStore();
  const [kind, setKind] = useState<DesignKind>("normal");
  const [list, setList, reload] = useLoad(() => api.designs(kind), [kind]);
  const [ver, setVer] = useState(0);
  const [edit, setEdit] = useState<number | null>(null);
  const bump = () => { setVer((v) => v + 1); reload(); touch(); };
  const ext = kind === "normal" || kind === "flag" ? "nhd" : "nhpd";
  return (
    <div className="content"><div className="page wide">
      <Card bodyClass="" actions={null}>
        <div className="row" style={{ padding: "8px 12px 0" }}>
          <Tabs value={kind} onChange={(k) => { setKind(k); setList(null); }} tabs={[
            { value: "normal", label: "My Designs" }, { value: "pro", label: "PRO Designs" }, { value: "tailor", label: "Able Sisters" }, { value: "flag", label: "Island flag" }]} />
          <span className="spacer" />
          <Button size="sm" icon={<UserCheck />} onClick={async () => {
            if (!(await confirm({ title: `Make ${save?.players[player]?.name} the author of every design?`, message: "Lets this resident edit designs that were imported from other islands." }))) return;
            const n = await api.setDesignAuthorAll(kind, player); bump(); toast("ok", `Updated ${n} designs`);
          }}>Claim all</Button>
        </div>
        <div style={{ padding: 16 }}>
          {!list ? <Loading /> : (
            <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(118px, 1fr))", gap: 14 }}>
              {list.map((d) => (
                <div key={d.index} className="col" style={{ gap: 6, cursor: "default" }} onClick={() => setEdit(d.index)}>
                  <img className="design-thumb" src={asset(`design/${kind}/${d.index}?r=${rev}.${ver}`)} alt="" style={{ opacity: d.empty ? 0.35 : 1 }} />
                  <div className="tiny" style={{ lineHeight: 1.25 }}>
                    <div className="bold ellipsis">{d.name || <span className="faint">(empty)</span>}</div>
                    <div className="faint ellipsis">{d.player_name ? `${d.player_name} · ${d.town_name}` : `#${d.index + 1}`}</div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </Card>
      {edit !== null && <DesignEditor kind={kind} index={edit} ext={ext} onClose={() => setEdit(null)} onSaved={bump} />}
    </div></div>
  );
}

type Tool = "pencil" | "fill" | "pick" | "erase";

function DesignEditor({ kind, index, ext, onClose, onSaved }: { kind: DesignKind; index: number; ext: string; onClose: () => void; onSaved: () => void }) {
  const { toast } = useStore();
  const [d, setD] = useState<DesignPixels | null>(null);
  const [sheet, setSheet] = useState(0);
  const [color, setColor] = useState(0);
  const [tool, setTool] = useState<Tool>("pencil");
  const [dirty, setDirty] = useState(false);
  const canvas = useRef<HTMLCanvasElement>(null);
  const painting = useRef(false);
  const load = () => api.designPixels(kind, index).then((x) => { setD(x); setDirty(false); });
  useEffect(() => { load(); }, [kind, index]);
  const sheets = d ? d.pixels.length / 1024 : 1;

  useEffect(() => {
    const c = canvas.current;
    if (!c || !d) return;
    const ctx = c.getContext("2d")!;
    const img = ctx.createImageData(32, 32);
    for (let i = 0; i < 1024; i++) {
      const p = d.pixels[sheet * 1024 + i];
      if (p >= 15) {
        const light = ((i % 32) >> 2) % 2 === ((i >> 5) >> 2) % 2;
        img.data.set(light ? [235, 235, 235, 255] : [205, 205, 205, 255], i * 4);
      } else img.data.set([...d.palette[p], 255], i * 4);
    }
    ctx.putImageData(img, 0, 0);
  }, [d, sheet]);

  if (!d) return null;
  const pos = (e: React.MouseEvent) => {
    const r = (e.target as HTMLCanvasElement).getBoundingClientRect();
    return [Math.floor(((e.clientX - r.left) / r.width) * 32), Math.floor(((e.clientY - r.top) / r.height) * 32)];
  };
  const apply = (x: number, y: number) => {
    if (x < 0 || y < 0 || x > 31 || y > 31) return;
    const base = sheet * 1024;
    const px = d.pixels.slice();
    const i = base + y * 32 + x;
    if (tool === "pick") { if (px[i] < 15) setColor(px[i]); setTool("pencil"); return; }
    const v = tool === "erase" ? 15 : color;
    if (tool === "fill") {
      const target = px[i];
      if (target === v) return;
      const stack = [[x, y]];
      while (stack.length) {
        const [cx, cy] = stack.pop()!;
        if (cx < 0 || cy < 0 || cx > 31 || cy > 31) continue;
        const k = base + cy * 32 + cx;
        if (px[k] !== target) continue;
        px[k] = v;
        stack.push([cx + 1, cy], [cx - 1, cy], [cx, cy + 1], [cx, cy - 1]);
      }
    } else {
      if (px[i] === v) return;
      px[i] = v;
    }
    setD({ ...d, pixels: px }); setDirty(true);
  };
  const save = async () => {
    await api.setDesignPixels(kind, index, d.info.name, d.palette, d.pixels);
    setDirty(false); onSaved(); toast("ok", "Design saved");
  };
  const hexc = (c: number[]) => "#" + c.map((x) => x.toString(16).padStart(2, "0")).join("");
  return (
    <Modal title={d.info.name || `Design ${index + 1}`} size="wide" onClose={onClose} icon={<Palette size={18} />} footer={<>
      <MenuButton label="Import / export" items={[
        { label: "Import PNG…", icon: <ImageUp />, onClick: async () => { const p = await pickFile("Import image", [F.png]); if (p) { try { await api.importDesignPng(kind, index, p); await load(); onSaved(); toast("ok", "Image imported", "Colors were reduced to a 15-color palette."); } catch (e) { toast("error", "Import failed", String(e)); } } } },
        { label: "Export PNG…", icon: <ImageDown />, onClick: async () => { const p = await saveFile("Export image", `${d.info.name || "design"}.png`, [F.png]); if (p) { await api.designPng(kind, index, p, 16); toast("ok", "Exported"); } } },
        "sep",
        { label: `Import .${ext}…`, icon: <FileUp />, onClick: async () => { const p = await pickFile("Import design", [ext === "nhd" ? F.nhd : F.nhpd, F.all]); if (p) { try { await api.importDesign(kind, index, p, true); await load(); onSaved(); } catch (e) { toast("error", "Import failed", String(e)); } } } },
        { label: `Export .${ext}…`, icon: <FileDown />, onClick: async () => { const p = await saveFile("Export design", `${d.info.name || "design"}.${ext}`, [ext === "nhd" ? F.nhd : F.nhpd]); if (p) { await api.exportDesign(kind, index, p); toast("ok", "Exported"); } } },
      ]} icon={<Download />} />
      <span className="spacer" />
      <Button onClick={onClose}>Close</Button>
      <Button variant="primary" disabled={!dirty} onClick={save}>Save design</Button>
    </>}>
      <div className="grid" style={{ gridTemplateColumns: "auto 1fr", gap: 20, alignItems: "start" }}>
        <div className="col" style={{ gap: 10 }}>
          {sheets > 1 && <Segmented value={sheet} onChange={setSheet} options={SHEETS.map((l, i) => ({ value: i, label: l }))} />}
          <canvas ref={canvas} width={32} height={32} className="pixel" style={{ width: 448, height: 448, borderRadius: 12, border: "1px solid var(--border)", cursor: "crosshair" }}
            onMouseDown={(e) => { painting.current = true; const [x, y] = pos(e); apply(x, y); }}
            onMouseMove={(e) => { if (painting.current && (tool === "pencil" || tool === "erase")) { const [x, y] = pos(e); apply(x, y); } }}
            onMouseUp={() => (painting.current = false)} onMouseLeave={() => (painting.current = false)} />
        </div>
        <div className="col" style={{ gap: 14 }}>
          <Field label="Name" hint="Up to 20 characters"><TextInput value={d.info.name} maxLength={20} onChange={(v) => { setD({ ...d, info: { ...d.info, name: v } }); setDirty(true); }} /></Field>
          <Field label="Tool">
            <Segmented value={tool} onChange={setTool} options={[
              { value: "pencil", label: "Draw", icon: <Pencil /> }, { value: "fill", label: "Fill", icon: <PaintBucket /> },
              { value: "pick", label: "Pick", icon: <Pipette /> }, { value: "erase", label: "Erase", icon: <Eraser /> }]} />
          </Field>
          <Field label={<><Droplet size={13} /> Palette</>} hint="Click to select · double-click the swatch below to change its color">
            <div style={{ display: "grid", gridTemplateColumns: "repeat(5, 36px)", gap: 6 }}>
              {d.palette.map((c, i) => (
                <button key={i} onClick={() => { setColor(i); if (tool === "erase" || tool === "pick") setTool("pencil"); }}
                  style={{ width: 36, height: 36, borderRadius: 9, background: hexc(c), border: i === color ? "3px solid var(--accent)" : "1px solid var(--border-strong)", boxShadow: i === color ? "0 0 0 3px var(--accent-soft)" : undefined }} />
              ))}
            </div>
          </Field>
          <div className="row">
            <input type="color" value={hexc(d.palette[color])} style={{ width: 48, height: 32, border: 0, background: "none" }}
              onChange={(e) => {
                const v = e.target.value; const rgb: [number, number, number] = [1, 3, 5].map((k) => parseInt(v.slice(k, k + 2), 16)) as [number, number, number];
                const pal = d.palette.slice(); pal[color] = rgb; setD({ ...d, palette: pal }); setDirty(true);
              }} />
            <span className="small muted">Color {color + 1} · {hexc(d.palette[color]).toUpperCase()}</span>
          </div>
          <div className="tiny faint">Author: {d.info.player_name || "—"} ({d.info.town_name || "—"})</div>
        </div>
      </div>
    </Modal>
  );
}
