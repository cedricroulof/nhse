import { Download, Eraser, Mail as MailIcon, MailPlus, Replace, Search, Star, Upload } from "lucide-react";
import { useMemo, useState } from "react";
import { ItemEditor } from "../components/ItemEditor";
import { Button, Card, Check, CountedInput, Empty, Field, Loading, Modal, NumberInput, Segmented, Select, TextInput } from "../components/ui";
import { DateInput, VillagerPicker } from "../components/Widgets";
import { api, NONE, type Mail } from "../lib/api";
import { F, pickFile, saveFile } from "../lib/dialog";
import { fmtDate, itemLabel, itemSprite } from "../lib/items";
import { useGD, useLoad, useStore } from "../lib/store";

const isUsed = (m: Mail) => m.status === 3 || m.status === 4;

export default function Mailbox() {
  const { player, touch, toast, save, confirm, settings } = useStore();
  const gd = useGD();
  const [box, setBox] = useLoad(() => api.playerMail(player), [player]);
  const [profile] = useLoad(() => api.playerProfile(player), [player]);
  const [sel, setSel] = useState<number | null>(null);
  const [draft, setDraft] = useState<Mail | null>(null);
  const [q, setQ] = useState("");
  const [replace, setReplace] = useState(false);
  const used = useMemo(() => (box ? box.mails.filter(isUsed).sort((a, b) => b.sort_index - a.sort_index) : []), [box]);
  const shown = used.filter((m) => !q || (m.sender_name + m.to_line + m.message + m.from_line).toLowerCase().includes(q.toLowerCase()));
  if (!box || !profile) return <Loading />;
  const open = (m: Mail) => { setSel(m.slot); setDraft({ ...m }); };
  const newLetter = () => {
    const free = box.mails.find((m) => !isUsed(m));
    if (!free) { toast("warn", "The mailbox is full (300 letters)"); return; }
    const today = new Date();
    setSel(free.slot);
    setDraft({
      ...free, status: 3, favorite: false, sender_type: 2, sender_species: 22, sender_variant: 0, sender_id: 0, sender_name: "",
      sender_island_id: profile.town_id, sender_island_name: profile.town_name,
      receiver_island_id: profile.town_id, receiver_island_name: profile.town_name, receiver_player_id: profile.player_id, receiver_player_name: profile.name,
      date: { year: today.getFullYear(), month: today.getMonth() + 1, day: today.getDate() }, item: NONE, wrapping: 0, is_letter: true,
      to_line: `Dear ${profile.name},`, message: "", from_line: "", stationery: 0, language: "en-US",
    });
  };
  const saveDraft = async () => {
    if (!draft) return;
    const m = { ...draft };
    if (m.sender_type === 0) { m.sender_id = profile.player_id; }
    if (m.sender_type === 1) m.sender_id = 2;
    if (m.sender_type === 2) m.sender_id = 0;
    const r = await api.setPlayerMail(player, m);
    setBox(r); touch(); toast("ok", "Letter saved");
    setDraft({ ...r.mails[m.slot] });
  };
  const villagerName = draft ? `${gd.enums.VillagerSpecies.find(([, v]) => v === draft.sender_species)?.[0] ?? "non"}${String(draft.sender_variant).padStart(2, "0")}` : "";
  const stationeryNames = gd.mail;

  return (
    <div className="content"><div className="page wide">
      <div className="grid" style={{ gridTemplateColumns: "340px 1fr", alignItems: "start" }}>
        <Card title={<span className="row"><MailIcon size={16} /> Mailbox</span>} desc={`${used.length} of 300 · ${save?.players[player]?.name}`}
          actions={<div className="row">
            <Button size="sm" iconOnly icon={<Replace />} title="Find & replace in all letters" onClick={() => setReplace(true)} />
            <Button size="sm" variant="primary" icon={<MailPlus />} onClick={newLetter}>New</Button>
          </div>}>
          <div className="input-wrap" style={{ marginBottom: 8 }}><Search /><input className="input" placeholder="Search letters…" value={q} onChange={(e) => setQ(e.target.value)} /></div>
          <div className="list scroll" style={{ maxHeight: "calc(100vh - 260px)" }}>
            {shown.length === 0 && <Empty title="No letters" />}
            {shown.map((m) => (
              <div key={m.slot} className={"list-item " + (sel === m.slot ? "sel" : "")} onClick={() => open(m)}>
                {m.item.id !== 0xfffe ? <img src={itemSprite(m.item.id)} style={{ width: 26, height: 26, objectFit: "contain" }} alt="" /> : <MailIcon size={20} color="var(--text-3)" />}
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div className="row" style={{ gap: 6 }}>
                    <span className={"ellipsis " + (m.status === 3 ? "bold" : "")}>{m.sender_type === 2 ? gd.npc_senders.find(([s]) => s === m.sender_species)?.[2] ?? m.sender_name : m.sender_name || "(unknown)"}</span>
                    {m.favorite && <Star size={12} color="var(--warn)" fill="var(--warn)" />}
                  </div>
                  <div className="tiny faint ellipsis">{fmtDate(m.date)} · {(m.message || m.to_line || "").replace(/\n/g, " ")}</div>
                </div>
              </div>
            ))}
          </div>
        </Card>

        {draft ? (
          <Card title={isUsed(box.mails[draft.slot]) ? "Edit letter" : "New letter"} desc={`Slot ${draft.slot + 1}`}
            actions={<div className="row">
              <Button size="sm" iconOnly icon={<Download />} title="Export (.nhpb)" onClick={async () => { const p = await saveFile("Export letter", `letter-${draft.slot + 1}.nhpb`, [F.nhpb]); if (p) { await api.exportPlayerMail(player, draft.slot, p); toast("ok", "Exported"); } }} />
              <Button size="sm" iconOnly icon={<Upload />} title="Import (.nhpb)" onClick={async () => { const p = await pickFile("Import letter", [F.nhpb, F.all]); if (p) { try { await api.importPlayerMail(player, draft.slot, p); const r = await api.playerMail(player); setBox(r); setDraft({ ...r.mails[draft.slot] }); touch(); } catch (e) { toast("error", "Import failed", String(e)); } } }} />
              <Button size="sm" variant="danger" iconOnly icon={<Eraser />} title="Delete letter" onClick={async () => {
                if (!(await confirm({ title: "Delete this letter?", danger: true, confirm: "Delete" }))) return;
                await api.clearPlayerMail(player, draft.slot); setBox(await api.playerMail(player)); setDraft(null); setSel(null); touch();
              }} />
            </div>}>
            <div className="col" style={{ gap: 14 }}>
              <div className="row wrap" style={{ gap: 14 }}>
                <Field label="From">
                  <Segmented value={draft.sender_type} onChange={(v) => setDraft({ ...draft, sender_type: v })} options={[{ value: 0, label: "Player" }, { value: 1, label: "Villager" }, { value: 2, label: "Special NPC" }]} />
                </Field>
                <Check label="Unread" checked={draft.status === 3} onChange={(v) => setDraft({ ...draft, status: v ? 3 : 4 })} />
                <Check label="Favorite" checked={draft.favorite} onChange={(v) => setDraft({ ...draft, favorite: v })} />
                <Field label="Date"><DateInput value={draft.date} onChange={(d) => setDraft({ ...draft, date: d })} /></Field>
              </div>
              <div className="grid cols-2" style={{ gap: 12 }}>
                {draft.sender_type === 0 && <>
                  <Field label="Sender name"><TextInput value={draft.sender_name} maxLength={10} onChange={(v) => setDraft({ ...draft, sender_name: v })} /></Field>
                  <Field label="Sender island"><TextInput value={draft.sender_island_name} maxLength={10} onChange={(v) => setDraft({ ...draft, sender_island_name: v })} /></Field>
                  <Field label="Language"><Select value={draft.language || "en-US"} onChange={(v) => setDraft({ ...draft, language: v })} options={gd.mail_languages.map((l) => ({ value: l, label: l }))} /></Field>
                </>}
                {draft.sender_type === 1 && <>
                  <Field label="Villager"><VillagerPicker value={villagerName} onChange={(n) => { const sp = gd.enums.VillagerSpecies.find(([k]) => k === n.slice(0, 3)); if (sp) setDraft({ ...draft, sender_species: sp[1], sender_variant: Number(n.slice(3)), sender_name: gd.villagers.find((v) => v.internal === n)?.name ?? draft.sender_name }); }} /></Field>
                  <Field label="Display name"><TextInput value={draft.sender_name} maxLength={10} onChange={(v) => setDraft({ ...draft, sender_name: v })} /></Field>
                </>}
                {draft.sender_type === 2 && (
                  <Field label="Character"><Select value={draft.sender_species} onChange={(v) => setDraft({ ...draft, sender_species: v, sender_variant: 0 })} options={gd.npc_senders.map(([s, , n]) => ({ value: s, label: n }))} /></Field>
                )}
                <Field label="Recipient name"><TextInput value={draft.receiver_player_name} maxLength={10} onChange={(v) => setDraft({ ...draft, receiver_player_name: v })} /></Field>
              </div>
              <div className="grid" style={{ gridTemplateColumns: "1fr 300px", gap: 16, alignItems: "start" }}>
                <div className="col" style={{ gap: 10 }}>
                  <Field label="Greeting line"><CountedInput max={47} value={draft.to_line} onChange={(v) => setDraft({ ...draft, to_line: v })} /></Field>
                  <Field label="Message"><CountedInput multiline rows={7} max={239} soft={168} value={draft.message} onChange={(v) => setDraft({ ...draft, message: v })} /></Field>
                  <Field label="Signature"><CountedInput max={47} value={draft.from_line} onChange={(v) => setDraft({ ...draft, from_line: v })} /></Field>
                </div>
                <div className="col" style={{ gap: 10 }}>
                  <Field label="Kind"><Segmented value={draft.is_letter ? 1 : 0} onChange={(v) => setDraft({ ...draft, is_letter: v === 1 })} options={[{ value: 1, label: "Letter" }, { value: 0, label: "Parcel" }]} /></Field>
                  {draft.is_letter ? (
                    <Field label="Stationery"><Select value={draft.stationery} onChange={(v) => setDraft({ ...draft, stationery: v })} options={gd.stationery.map((code, i) => ({ value: code, label: stationeryNames.find(([k]) => k === `mail${String(i).padStart(2, "0")}`)?.[1] ?? `Stationery ${i}` }))} /></Field>
                  ) : (
                    <Field label="Box"><Select value={draft.wrapping} onChange={(v) => setDraft({ ...draft, wrapping: v })} options={gd.box_skins.map(([v, l]) => ({ value: v, label: l }))} /></Field>
                  )}
                  <div className="label">Attached item</div>
                  <div className="card card-pad" style={{ background: "var(--card-2)" }}>
                    <ItemEditor item={draft.item} allowWrap={false} compact onChange={(it) => setDraft({ ...draft, item: it })} />
                  </div>
                  {settings?.advanced && <Field label="Sort index"><NumberInput value={draft.sort_index} max={65535} onChange={(v) => setDraft({ ...draft, sort_index: v })} /></Field>}
                </div>
              </div>
              <div className="row" style={{ justifyContent: "flex-end" }}>
                <span className="small faint" style={{ marginRight: "auto" }}>{draft.item.id !== 0xfffe ? `Attachment: ${itemLabel(gd, draft.item)}` : ""}</span>
                <Button onClick={() => { const m = box.mails[draft.slot]; setDraft(isUsed(m) ? { ...m } : null); }}>Discard edits</Button>
                <Button variant="primary" onClick={saveDraft}>Save letter</Button>
              </div>
            </div>
          </Card>
        ) : <Card><Empty icon={<MailIcon />} title="Select a letter" >or write a new one</Empty></Card>}
      </div>
      {replace && <ReplaceModal onClose={() => setReplace(false)} onDone={async () => { setBox(await api.playerMail(player)); setDraft(null); }} />}
    </div></div>
  );
}

function ReplaceModal({ onClose, onDone }: { onClose: () => void; onDone: () => void }) {
  const { player, touch, toast } = useStore();
  const [find, setFind] = useState("");
  const [rep, setRep] = useState("");
  const [names, setNames] = useState(true);
  const [text, setText] = useState(true);
  return (
    <Modal title="Find & replace in letters" onClose={onClose} footer={<>
      <Button onClick={onClose}>Cancel</Button>
      <Button variant="primary" disabled={!find} onClick={async () => {
        const n = await api.replaceInMail(player, find, rep, names, text);
        if (n) touch();
        toast(n ? "ok" : "info", n ? `Replaced ${n} occurrence${n === 1 ? "" : "s"}` : "No matches found");
        onDone(); onClose();
      }}>Replace all</Button>
    </>}>
      <div className="col" style={{ gap: 12 }}>
        <div className="small muted">Useful after renaming your character: letters keep the old name as plain text.</div>
        <Field label="Find"><TextInput value={find} onChange={setFind} autoFocus /></Field>
        <Field label="Replace with"><TextInput value={rep} onChange={setRep} /></Field>
        <div className="row"><Check label="Names (sender, recipient, islands)" checked={names} onChange={setNames} /><Check label="Letter text" checked={text} onChange={setText} /></div>
      </div>
    </Modal>
  );
}
