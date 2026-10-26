import { FileDown, FileUp, ShieldCheck } from "lucide-react";
import { useState } from "react";
import { Button, Callout, Card } from "../components/ui";
import { api, type InvalidHash } from "../lib/api";
import { pickFolder } from "../lib/dialog";
import { useStore } from "../lib/store";

export default function Tools() {
  const { toast, touch, run } = useStore();
  const [hashes, setHashes] = useState<InvalidHash[] | null>(null);
  return (
    <div className="content"><div className="page">
      <Card title={<span className="row"><ShieldCheck size={16} /> Verify hashes</span>} desc="Checks every checksum of the save as it is in memory right now">
        <div className="col">
          <div className="small muted">Saving always recomputes every hash, so mismatches here only mean the data was changed since it was loaded (or the dump was already damaged).</div>
          <div><Button onClick={async () => setHashes(await api.verifyHashes())}>Verify now</Button></div>
          {hashes && (hashes.length === 0 ? <Callout kind="ok" title="All hashes are valid" /> : (
            <Callout kind="warn" title={`${hashes.length} region(s) differ`}>
              <div className="mono tiny" style={{ maxHeight: 200, overflow: "auto" }}>{hashes.map((h, i) => <div key={i}>{h.file} · {h.region}</div>)}</div>
            </Callout>
          ))}
        </div>
      </Card>
      <Card title="Decrypted files" desc="For research and use with other tools">
        <div className="col">
          <div className="small muted">Export writes every file decrypted (same folder layout). Import loads previously exported decrypted files back into this session; sizes must match.</div>
          <div className="row">
            <Button icon={<FileDown />} onClick={async () => {
              const dir = await pickFolder("Export decrypted files to…");
              if (dir) await run(async () => { const n = await api.dumpDecrypted(dir); toast("ok", `Exported ${n} files`, dir); });
            }}>Export decrypted…</Button>
            <Button icon={<FileUp />} onClick={async () => {
              const dir = await pickFolder("Import decrypted files from…");
              if (dir) await run(async () => { const n = await api.loadDecrypted(dir); touch(); toast("ok", `Imported ${n} files`); });
            }}>Import decrypted…</Button>
          </div>
        </div>
      </Card>
    </div></div>
  );
}
