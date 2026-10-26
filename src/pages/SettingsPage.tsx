import { Image, Languages, Moon, Monitor, ShieldCheck, Sun, Trash2, Wrench } from "lucide-react";
import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Button, Card, Field, NumberInput, Segmented, Select, ToggleRow } from "../components/ui";
import { setSpriteEpoch } from "../lib/items";
import { api } from "../lib/api";
import { clearItemInfoCache } from "../components/ItemEditor";
import { last } from "../lib/util";
import { useStore } from "../lib/store";

export default function SettingsPage() {
  const { settings, updateSettings, info, toast } = useStore();
  const [dl, setDl] = useState<{ done: number; total: number | null } | null>(null);
  const [hasPack, setHasPack] = useState(info?.has_image_pack ?? false);
  useEffect(() => {
    const un = listen<{ done: number; total: number | null }>("image-progress", (e) => setDl(e.payload));
    return () => { un.then((f) => f()); };
  }, []);
  if (!settings || !info) return null;
  return (
    <div className="content"><div className="page" style={{ maxWidth: 760 }}>
      <Card title="Appearance">
        <div className="col" style={{ gap: 14 }}>
          <Field label="Theme">
            <Segmented value={settings.theme} onChange={(theme) => updateSettings({ theme })} options={[
              { value: "system", label: "System", icon: <Monitor /> }, { value: "light", label: "Light", icon: <Sun /> }, { value: "dark", label: "Dark", icon: <Moon /> }]} />
          </Field>
          <Field label={<><Languages size={13} /> Game text language</>} hint="Language used for item, villager and other in-game names">
            <Select value={settings.language} onChange={(language) => { clearItemInfoCache(); updateSettings({ language }); }} options={info.languages.map(([v, l]) => ({ value: v, label: l }))} style={{ width: 220 }} />
          </Field>
        </div>
      </Card>
      <Card title={<span className="row"><ShieldCheck size={16} /> Backups</span>}>
        <ToggleRow title="Back up when opening a save" desc="Stores a compressed copy of just the save files (not the surrounding folder)." on={settings.backup_on_open} onChange={(v) => updateSettings({ backup_on_open: v })} />
        <Field label="Backups to keep per island" hint="Older automatic backups are deleted. 0 keeps everything.">
          <NumberInput value={settings.backup_keep} max={1000} width={120} onChange={(v) => updateSettings({ backup_keep: v })} />
        </Field>
      </Card>
      <Card title={<span className="row"><Image size={16} /> Item images</span>} desc="Built-in icons are always available. A community image pack adds a picture for every item.">
        <div className="col">
          {dl && <div className="col" style={{ gap: 4 }}><div className="progress"><div style={{ width: dl.total ? `${(dl.done / dl.total) * 100}%` : "30%" }} /></div>
            <div className="tiny faint">{(dl.done / 1e6).toFixed(1)} MB{dl.total ? ` of ${(dl.total / 1e6).toFixed(1)} MB` : ""}</div></div>}
          <div className="row">
            <Button variant="primary" disabled={!!dl} onClick={async () => {
              try { const n = await api.downloadImages(); setSpriteEpoch(Date.now()); setHasPack(true); toast("ok", `Installed ${n.toLocaleString()} item images`); }
              catch (e) { toast("error", "Download failed", String(e)); }
              finally { setDl(null); }
            }}>{hasPack ? "Re-download image pack" : "Download image pack (~90 MB)"}</Button>
            {hasPack && <Button variant="danger" icon={<Trash2 />} onClick={async () => { await api.removeImages(); setSpriteEpoch(Date.now()); setHasPack(false); toast("ok", "Image pack removed"); }}>Remove</Button>}
          </div>
        </div>
      </Card>
      <Card title={<span className="row"><Wrench size={16} /> Editing</span>}>
        <ToggleRow title="Advanced mode" desc="Shows raw values, hex ids and event flag editors." on={settings.advanced} onChange={(v) => updateSettings({ advanced: v })} />
        <ToggleRow title="Map: handle multi-tile items automatically" desc="Creates and removes the extension tiles of large items." on={settings.map_auto_extensions} onChange={(v) => updateSettings({ map_auto_extensions: v })} />
        <ToggleRow title="Map: prevent overlapping items" desc="Refuses to place items on occupied tiles." on={settings.map_prevent_overlap} onChange={(v) => updateSettings({ map_prevent_overlap: v })} />
        <ToggleRow title="Rooms: allow items to overhang the room edge" desc="The game accepts furniture partially outside the 10×10 grid (e.g. rugs by the door)." on={settings.room_allow_overhang} onChange={(v) => updateSettings({ room_allow_overhang: v })} />
      </Card>
      <div className="center tiny faint">NHSE {info.version} · supports game versions {info.supported_versions[0]} – {last(info.supported_versions)}</div>
    </div></div>
  );
}
