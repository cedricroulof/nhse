import { ArchiveRestore, FolderOpen, History, Plus, Trash2 } from "lucide-react";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { Button, Callout, Card, Empty, Loading } from "../components/ui";
import { api } from "../lib/api";
import { fmtBytes } from "../lib/items";
import { useLoad, useStore } from "../lib/store";

export default function Backups() {
  const { info, save, open, confirm, toast, dirty } = useStore();
  const [list, , reload] = useLoad(api.listBackups, []);
  if (!list) return <Loading />;
  const total = list.reduce((a, b) => a + b.size, 0);
  return (
    <div className="content"><div className="page">
      <Callout kind="info" title="How backups work">
        Only the save files themselves are backed up — compressed into one ZIP per backup in the app's data folder — whenever a save is opened.
        Identical backups are skipped and the oldest ones are pruned automatically (configurable in Settings).
      </Callout>
      <Card title={<span className="row"><History size={16} /> Backups</span>} desc={`${list.length} backups · ${fmtBytes(total)}`}
        actions={<div className="row">
          {info && <Button size="sm" icon={<FolderOpen />} onClick={() => revealItemInDir(info.backups_dir + "/.")}>Show folder</Button>}
          {save && <Button size="sm" icon={<Plus />} onClick={async () => { const r = await api.createBackup(); toast("ok", r ? "Backup created" : "An identical backup already exists"); reload(); }}>Back up now</Button>}
        </div>}>
        {list.length === 0 ? <Empty icon={<History />} title="No backups yet" /> : (
          <table className="table">
            <thead><tr><th>Name</th><th>Date</th><th>Size</th><th style={{ width: 220 }}></th></tr></thead>
            <tbody>
              {list.map((b) => (
                <tr key={b.path}>
                  <td className="ellipsis" style={{ maxWidth: 380 }}>{b.name.replace(/\s*\[[0-9a-f]+\]\.zip$/, "")}</td>
                  <td className="nowrap">{b.modified}</td>
                  <td className="nowrap">{fmtBytes(b.size)}</td>
                  <td>
                    <div className="row" style={{ justifyContent: "flex-end" }}>
                      <Button size="sm" icon={<ArchiveRestore />} title="Open this backup (edits are saved into the backup archive)" onClick={async () => {
                        if (dirty && !(await confirm({ title: "Open backup?", message: "Unsaved changes in the current save will be lost.", danger: true }))) return;
                        open(b.path);
                      }}>Open</Button>
                      <Button size="sm" iconOnly icon={<FolderOpen />} onClick={() => revealItemInDir(b.path)} />
                      <Button size="sm" iconOnly variant="danger" icon={<Trash2 />} onClick={async () => {
                        if (await confirm({ title: "Delete backup?", message: b.name, danger: true, confirm: "Delete" })) { await api.deleteBackup(b.path); reload(); }
                      }} />
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </Card>
    </div></div>
  );
}
