import { Download, FileText, StickyNote, Upload } from "lucide-react";
import { useState } from "react";
import { Button, Card, CountedInput, Empty, Field, Loading } from "../components/ui";
import { DateInput } from "../components/Widgets";
import { api, type Board, type Post } from "../lib/api";
import { F, pickFile, saveFile } from "../lib/dialog";
import { asset, fmtDate } from "../lib/items";
import { useLoad, useStore } from "../lib/store";

export default function Bulletin() {
  const { touch, toast, rev } = useStore();
  const [b, setB] = useLoad(api.bulletin, []);
  const [sel, setSel] = useState<number | null>(null);
  if (!b) return <Loading />;
  const used = b.posts.filter((p) => p.body || p.has_handwriting || p.unique_id);
  const cur = sel !== null ? b.posts[sel] : used[0];
  const upd = (post: Post) => {
    const next: Board = { ...b, posts: b.posts.map((p) => (p.slot === post.slot ? post : p)) };
    setB(next); api.setBulletin(next).then(touch);
  };
  return (
    <div className="content"><div className="page">
      <Card title={<span className="row"><StickyNote size={16} /> Bulletin board</span>} desc={`${used.length} post${used.length === 1 ? "" : "s"} · built ${fmtDate(b.built)}`}
        actions={<Button size="sm" icon={<FileText />} onClick={async () => {
          const p = await saveFile("Export posts as text", "bulletin-board.txt", [F.txt]);
          if (p) { const n = await api.exportBulletinText(p); toast("ok", `Exported ${n} posts`); }
        }}>Export all as text</Button>}>
        {used.length === 0 ? <Empty icon={<StickyNote />} title="No posts on the board" /> : (
          <div className="grid" style={{ gridTemplateColumns: "300px 1fr", alignItems: "start" }}>
            <div className="list">
              {used.map((p) => (
                <div key={p.slot} className={"list-item " + (cur?.slot === p.slot ? "sel" : "")} onClick={() => setSel(p.slot)}>
                  <div style={{ minWidth: 0 }}>
                    <div className="bold ellipsis">{p.body.split("\n")[0] || (p.has_handwriting ? "(drawing)" : "(empty)")}</div>
                    <div className="tiny faint">{fmtDate(p.date)} · {p.author.player_name || "?"}</div>
                  </div>
                </div>
              ))}
            </div>
            {cur && (
              <div className="col" style={{ gap: 12 }}>
                {cur.has_handwriting && (
                  <div className="card" style={{ padding: 8, background: "#f4eedb" }}>
                    <img className="pixel" src={asset(`handwriting/${cur.slot}?r=${rev}`)} style={{ width: "100%", maxWidth: 520, display: "block" }} alt="handwriting" />
                    <div className="tiny" style={{ color: "#8a7a55", marginTop: 4 }}>Handwriting preview</div>
                  </div>
                )}
                <Field label="Message"><CountedInput multiline rows={6} max={240} value={cur.body} onChange={(v) => upd({ ...cur, body: v })} /></Field>
                <Field label="Signature"><CountedInput max={48} value={cur.footer} onChange={(v) => upd({ ...cur, footer: v })} /></Field>
                <div className="row" style={{ alignItems: "flex-end" }}>
                  <Field label="Date"><DateInput value={cur.date} onChange={(d) => upd({ ...cur, date: d })} /></Field>
                  <span className="spacer" />
                  <Button size="sm" icon={<Download />} onClick={async () => { const p = await saveFile("Export post", `post-${cur.slot + 1}.nhbp`, [F.nhbp]); if (p) { await api.exportBulletinPost(cur.slot, p); toast("ok", "Post exported"); } }}>Export</Button>
                  <Button size="sm" icon={<Upload />} onClick={async () => { const p = await pickFile("Import post", [F.nhbp, F.all]); if (p) { try { await api.importBulletinPost(cur.slot, p); touch(); setB(await api.bulletin()); toast("ok", "Post imported"); } catch (e) { toast("error", "Import failed", String(e)); } } }}>Import</Button>
                </div>
                <div className="tiny faint">Posted by {cur.author.player_name} from {cur.author.town_name}</div>
              </div>
            )}
          </div>
        )}
      </Card>
    </div></div>
  );
}
