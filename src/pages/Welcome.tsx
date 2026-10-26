import { Archive, FileArchive, FolderOpen, HardDriveDownload, Leaf, ShieldCheck, Sparkles } from "lucide-react";
import { useEffect, useState } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import logo from "../logo.png";
import { Button, Card } from "../components/ui";
import { F, pickFile, pickFolder } from "../lib/dialog";
import { last } from "../lib/util";
import { useStore } from "../lib/store";

export function Welcome() {
  const { open, settings, info, gd } = useStore();
  const [over, setOver] = useState(false);

  useEffect(() => {
    let un: (() => void) | undefined;
    getCurrentWebview().onDragDropEvent((e) => {
      if (e.payload.type === "over") setOver(true);
      else if (e.payload.type === "leave") setOver(false);
      else if (e.payload.type === "drop") {
        setOver(false);
        const p = e.payload.paths[0];
        if (p) open(p);
      }
    }).then((f) => { un = f; });
    return () => un?.();
  }, [open]);

  return (
    <div className="welcome drag">
      <div className="welcome-inner no-drag">
        <Card className="hero" bodyClass="">
          <div className="col" style={{ gap: 18 }}>
            <img src={logo} className="logo" alt="" />
            <div>
              <h1>NHSE</h1>
              <div className="faint">Animal Crossing: New Horizons save editor · v{info?.version}</div>
            </div>
            <p className="lead">Edit your island, villagers, items, designs and more. Open a decrypted save dump from your Switch — a folder containing <code>main.dat</code> and the <code>Villager0…7</code> folders, or a ZIP of it.</p>
            <div className={"dropzone " + (over ? "over" : "")}>
              <HardDriveDownload />
              <div className="bold">Drop a save folder, main.dat or .zip here</div>
              <div className="row">
                <Button variant="primary" icon={<FolderOpen />} disabled={!gd} onClick={async () => { const p = await pickFolder("Open save folder"); if (p) open(p); }}>Open folder…</Button>
                <Button icon={<FileArchive />} disabled={!gd} onClick={async () => { const p = await pickFile("Open save", [F.zip, F.main]); if (p) open(p); }}>Open ZIP / main.dat…</Button>
              </div>
            </div>
            <div className="row small faint" style={{ gap: 14, flexWrap: "wrap" }}>
              <span className="row" style={{ gap: 5 }}><ShieldCheck size={14} /> Automatic, compact backups</span>
              <span className="row" style={{ gap: 5 }}><Sparkles size={14} /> Supports 1.0.0 – {last(info?.supported_versions)}</span>
            </div>
          </div>
        </Card>
        <div className="col" style={{ gap: 16 }}>
          <Card title="Recent">
            {settings?.recent.length ? (
              <div className="list">
                {settings.recent.map((r) => (
                  <div key={r} className="recent-item" onClick={() => open(r)}>
                    {r.toLowerCase().endsWith(".zip") ? <Archive size={18} color="var(--accent)" /> : <FolderOpen size={18} color="var(--accent)" />}
                    <div style={{ minWidth: 0, flex: 1 }}>
                      <div className="bold ellipsis">{last(r.split(/[\\/]/).filter(Boolean))}</div>
                      <div className="p">{r}</div>
                    </div>
                  </div>
                ))}
              </div>
            ) : <div className="faint small">Saves you open will show up here.</div>}
          </Card>
          <Card title="Good to know">
            <div className="col small muted" style={{ gap: 8 }}>
              <div className="row" style={{ alignItems: "flex-start" }}><Leaf size={14} style={{ marginTop: 2, flex: "none", color: "var(--accent)" }} /> NHSE edits save data dumped from the console; it does not dump or inject saves itself.</div>
              <div className="row" style={{ alignItems: "flex-start" }}><Leaf size={14} style={{ marginTop: 2, flex: "none", color: "var(--accent)" }} /> Each player must have saved at least once on the current game version.</div>
              <div className="row" style={{ alignItems: "flex-start" }}><Leaf size={14} style={{ marginTop: 2, flex: "none", color: "var(--accent)" }} /> Please play fair — don't cheat at the expense of others.</div>
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
}
