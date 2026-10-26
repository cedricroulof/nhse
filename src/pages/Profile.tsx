import { Banknote, Camera, Coins, Download, Gem, Hotel, Ticket, User } from "lucide-react";
import { useState } from "react";
import { Button, Callout, Card, Check, Field, Loading, NumberInput, Select, TextInput } from "../components/ui";
import { DateInput, MonthDay, fruitOptions } from "../components/Widgets";
import { api, type PlayerProfile } from "../lib/api";
import { F, saveFile } from "../lib/dialog";
import { useGD, useLoad, useStore } from "../lib/store";

export default function Profile() {
  const { player, touch, toast, refreshSave } = useStore();
  const gd = useGD();
  const [p, setP] = useLoad(() => api.playerProfile(player), [player]);
  const [photo] = useLoad(() => api.playerPhoto(player), [player]);
  const [name, setName] = useState<string | null>(null);
  const [town, setTown] = useState<string | null>(null);
  if (!p) return <Loading />;

  const commit = async (next: PlayerProfile) => {
    try {
      const r = await api.setPlayerProfile(player, next);
      setP(r);
      touch();
    } catch (e) { toast("error", "Could not apply", String(e)); }
  };
  const upd = (patch: Partial<PlayerProfile>) => commit({ ...p, ...patch });
  const rename = async (field: "name" | "town_name", value: string) => {
    if (value === p[field] || !value.trim()) return;
    await commit({ ...p, [field]: value });
    await refreshSave();
    toast("ok", field === "name" ? "Player renamed" : "Island renamed", "The new name was applied everywhere it is referenced (villager memories, designs, letters…).");
  };

  return (
    <div className="content">
      <div className="page">
        {p.invalid_values.length > 0 && <Callout kind="warn" title="Some values failed their checksum">{p.invalid_values.join(", ")} — the save may already be damaged. Editing them will rewrite a valid value.</Callout>}
        <div className="grid" style={{ gridTemplateColumns: "260px 1fr", alignItems: "start" }}>
          <Card bodyClass="card-body col" style={{ alignItems: "stretch" }}>
            <div style={{ aspectRatio: "4/3", borderRadius: 12, overflow: "hidden", background: "var(--card-2)", display: "grid", placeItems: "center", border: "1px solid var(--border)" }}>
              {photo ? <img src={photo} style={{ width: "100%", height: "100%", objectFit: "cover" }} alt="" /> : <Camera size={30} color="var(--text-3)" />}
            </div>
            <div className="center" style={{ marginTop: 6 }}>
              <div className="bold" style={{ fontSize: 16 }}>{p.name}</div>
              <div className="faint small">{p.town_name} · {p.dir}</div>
            </div>
            {photo && <Button size="sm" icon={<Download />} onClick={async () => {
              const path = await saveFile("Export passport photo", `${p.name}.jpg`, [F.jpg]);
              if (path) { await api.exportPlayerPhoto(player, path); toast("ok", "Photo exported"); }
            }}>Export photo</Button>}
          </Card>

          <div className="col" style={{ gap: 16 }}>
            <Card title={<span className="row"><User size={16} /> Identity</span>} desc="Renaming updates every reference across the save">
              <div className="grid cols-2" style={{ gap: 12 }}>
                <Field label="Player name" hint="Up to 10 characters">
                  <TextInput value={name ?? p.name} maxLength={10} onChange={setName} onCommit={() => { if (name !== null) { rename("name", name); setName(null); } }} />
                </Field>
                <Field label="Island name" hint="Up to 10 characters">
                  <TextInput value={town ?? p.town_name} maxLength={10} onChange={setTown} onCommit={() => { if (town !== null) { rename("town_name", town); setTown(null); } }} />
                </Field>
                <Field label="Birthday"><MonthDay month={p.birthday.month || 1} day={p.birthday.day || 1} onChange={(m, d) => upd({ birthday: { month: m, day: d }, profile_birthday: { month: m, day: d } })} /></Field>
                <Field label="Passport fruit"><Select value={p.profile_fruit} onChange={(v) => upd({ profile_fruit: v })} options={fruitOptions(gd)} /></Field>
                <Field label="Passport registered"><DateInput value={p.profile_timestamp} onChange={(d) => upd({ profile_timestamp: d })} /></Field>
                <Field label="Role"><Check label="Island founder (made the island)" checked={p.profile_made_village} onChange={(v) => upd({ profile_made_village: v })} /></Field>
              </div>
            </Card>

            <Card title={<span className="row"><Coins size={16} /> Currencies</span>}>
              <div className="grid cols-2" style={{ gap: 12 }}>
                <Field label={<><Banknote size={13} /> Wallet</>} hint={p.wallet > 99999 ? "The game caps the wallet at 99,999" : "Max 99,999"} hintKind={p.wallet > 99999 ? "warn" : undefined}>
                  <NumberInput value={p.wallet} warnAbove={99999} onChange={(v) => upd({ wallet: v })} suffix="bells" />
                </Field>
                <Field label={<><Banknote size={13} /> Savings</>} hint="Max 999,999,999" hintKind={p.bank > 999999999 ? "warn" : undefined}>
                  <NumberInput value={p.bank} warnAbove={999999999} onChange={(v) => upd({ bank: v })} suffix="bells" />
                </Field>
                <Field label={<><Gem size={13} /> Nook Miles</>}><NumberInput value={p.nook_miles} onChange={(v) => upd({ nook_miles: v })} /></Field>
                <Field label={<><Gem size={13} /> Total Nook Miles earned</>}><NumberInput value={p.total_nook_miles} onChange={(v) => upd({ total_nook_miles: v })} /></Field>
                {p.hotel_tickets !== null && <Field label={<><Hotel size={13} /> Hotel tickets</>}><NumberInput value={p.hotel_tickets} onChange={(v) => upd({ hotel_tickets: v })} /></Field>}
                {p.poki !== null && <Field label={<><Ticket size={13} /> Poki (Happy Home Paradise)</>}><NumberInput value={p.poki} onChange={(v) => upd({ poki: v })} /></Field>}
              </div>
            </Card>

            <Card title="Inventory capacity" desc="How many slots are unlocked in game">
              <div className="grid cols-3" style={{ gap: 12 }}>
                <Field label="Pocket (first page)" hint="Normally 20"><NumberInput value={p.pocket_count} max={20} onChange={(v) => upd({ pocket_count: v })} /></Field>
                <Field label="Bag (second page)" hint="0, 10 or 20"><NumberInput value={p.bag_count} max={20} onChange={(v) => upd({ bag_count: v })} /></Field>
                <Field label="Home storage" hint={`Up to ${p.storage_capacity.toLocaleString()}`}><NumberInput value={p.storage_count} max={p.storage_capacity} onChange={(v) => upd({ storage_count: v })} /></Field>
              </div>
            </Card>
          </div>
        </div>
      </div>
    </div>
  );
}
