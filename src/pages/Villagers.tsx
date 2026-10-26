import { DoorOpen, Download, Heart, House, LogOut, MessageSquareQuote, RefreshCcw, Shirt, Sofa, Sparkles, Upload, UserCog, Wand2 } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { FlagTable } from "../components/FlagTable";
import { ItemEditor } from "../components/ItemEditor";
import { ItemGridEditor } from "../components/ItemGridEditor";
import { Badge, Button, Callout, Card, CountedInput, Field, Loading, Modal, NumberInput, Select, Tabs, TextInput, ToggleRow } from "../components/ui";
import { VillagerPicker, enumOptions, prettify } from "../components/Widgets";
import { api, type RoomFloorWall, type Villager, type VillagerHouse } from "../lib/api";
import { F, pickFile, saveFile } from "../lib/dialog";
import { villagerSprite } from "../lib/items";
import { useGD, useLoad, useStore } from "../lib/store";

type Tab = "general" | "memories" | "items" | "home" | "advanced";
const EMPTY = 35;

export default function Villagers() {
  const { touch, toast, confirm, save, settings } = useStore();
  const gd = useGD();
  const [list, setList] = useLoad(api.villagers, []);
  const [houses, setHouses] = useLoad(api.villagerHouses, []);
  const [sel, setSel] = useState(0);
  const [tab, setTab] = useState<Tab>("general");
  const [replacing, setReplacing] = useState(false);
  const timer = useRef<number | null>(null);
  if (!list || !houses) return <Loading />;
  const v = list[sel];
  const name = (x: Villager) => (x.species === EMPTY ? "Empty slot" : gd.villagers.find((g) => g.internal === x.internal_name)?.name ?? x.internal_name);

  const update = (next: Villager, immediate = false) => {
    setList(list.map((x) => (x.index === next.index ? next : x)));
    touch();
    if (timer.current) window.clearTimeout(timer.current);
    const go = () => api.setVillager(next).catch((e) => toast("error", "Could not save villager", String(e)));
    if (immediate) go(); else timer.current = window.setTimeout(go, 350);
  };
  const house = houses.find((h) => h.npc1 === sel) ?? houses[sel];
  const updHouse = (h: VillagerHouse) => { setHouses(houses.map((x) => (x.index === h.index ? h : x))); api.setVillagerHouse(h).then(touch); };

  return (
    <div className="content"><div className="page wide">
      <div style={{ display: "grid", gridTemplateColumns: "repeat(10, minmax(0,1fr))", gap: 10 }}>
        {list.map((x) => (
          <button key={x.index} className={"vcard " + (x.index === sel ? "sel" : "")} onClick={() => setSel(x.index)}>
            {x.moving_out && <span className="flag"><Badge kind="warn">moving</Badge></span>}
            <img src={villagerSprite(x.internal_name)} className="portrait" alt="" style={{ opacity: x.species === EMPTY ? 0.3 : 1 }} />
            <div className="n ellipsis" style={{ maxWidth: "100%" }}>{name(x)}</div>
            <div className="p">{x.species === EMPTY ? "—" : prettify((gd.enums.VillagerPersonality ?? []).find(([, p]) => p === x.personality)?.[0] ?? "")}</div>
          </button>
        ))}
      </div>

      <div className="row wrap">
        <Button size="sm" icon={<Heart />} onClick={async () => { setList(await api.maxFriendshipAll()); touch(); toast("ok", "Friendship maxed with every villager"); }}>Max friendship (all)</Button>
        <Button size="sm" icon={<LogOut />} onClick={async () => {
          if (!(await confirm({ title: "Ask every villager to move out?", message: "Each villager is flagged as moving out; they leave the next day in game." }))) return;
          for (const x of list) if (x.species !== EMPTY) await api.setVillager({ ...x, moving_out: true });
          setList(await api.villagers()); touch();
        }}>Move out all</Button>
      </div>

      <Card bodyClass="">
        <div className="row" style={{ padding: "16px 18px 0", gap: 16 }}>
          <img src={villagerSprite(v.internal_name)} className="portrait lg" alt="" />
          <div style={{ flex: 1, minWidth: 0 }}>
            <h1>{name(v)}</h1>
            <div className="muted">{v.internal_name} · slot {v.index + 1}{v.format < 2 && " · pre-2.0 format"}</div>
          </div>
          <Button icon={<Wand2 />} variant="primary" onClick={() => setReplacing(true)} disabled={save!.villager_format < 2}>Replace villager…</Button>
          <Button iconOnly icon={<Download />} title="Export villager (.nhv2)" onClick={async () => {
            const p = await saveFile("Export villager", `${name(v)}.${v.format >= 2 ? "nhv2" : "nhv"}`, [F.nhv]);
            if (p) { await api.exportVillager(v.index, p); toast("ok", "Villager exported"); }
          }} />
          <Button iconOnly icon={<Upload />} title="Import villager (.nhv2 / .nhv)" onClick={async () => {
            const p = await pickFile("Import villager", [F.nhv, F.all]);
            if (!p) return;
            try {
              let r = await api.importVillager(v.index, p, false);
              if (r.foreign && (await confirm({ title: "This villager comes from another island", message: `Its memories belong to ${r.owner || "another player"}. Rewrite them so it remembers your island and representative?`, confirm: "Adopt", cancel: "Keep as is" }))) {
                r = await api.importVillager(v.index, p, true);
              }
              setList(await api.villagers()); touch(); toast("ok", "Villager imported");
            } catch (e) { toast("error", "Import failed", String(e)); }
          }} />
        </div>
        <div style={{ padding: "8px 10px 0" }}>
          <Tabs value={tab} onChange={setTab} tabs={[
            { value: "general", label: "General", icon: <UserCog /> }, { value: "memories", label: "Memories", icon: <MessageSquareQuote /> },
            { value: "items", label: "Clothes & furniture", icon: <Shirt /> }, { value: "home", label: "House", icon: <House /> },
            ...(settings?.advanced ? [{ value: "advanced" as Tab, label: "Advanced", icon: <Sparkles /> }] : [])]} />
        </div>
        <div className="card-body">
          {tab === "general" && <General v={v} update={update} />}
          {tab === "memories" && <Memories v={v} update={update} />}
          {tab === "items" && (
            <div className="col" style={{ gap: 16 }}>
              <ItemGridEditor title={<span className="row"><Shirt size={16} /> Wardrobe</span>} items={v.wear} columns={8} slotSize={50} allowWrap={false} fileName={`${name(v)}-wear`}
                onChange={(wear) => update({ ...v, wear }, true)} />
              <ItemGridEditor title={<span className="row"><Sofa size={16} /> Furniture stock</span>} items={v.furniture} columns={8} slotSize={50} allowWrap={false} fileName={`${name(v)}-furniture`}
                onChange={(furniture) => update({ ...v, furniture }, true)} />
            </div>
          )}
          {tab === "home" && house && <HomeTab v={v} house={house} updHouse={updHouse} update={update} />}
          {tab === "advanced" && (
            <div className="col" style={{ gap: 16 }}>
              <div className="grid cols-4" style={{ gap: 12 }}>
                <Field label="Birth type"><NumberInput value={v.birth_type} max={255} onChange={(x) => update({ ...v, birth_type: x })} /></Field>
                <Field label="Inducement type"><NumberInput value={v.inducement_type} max={255} onChange={(x) => update({ ...v, inducement_type: x })} /></Field>
                <Field label="Move type"><NumberInput value={v.move_type} max={255} onChange={(x) => update({ ...v, move_type: x, moving_out: (x & 2) !== 0 })} /></Field>
                <Field label="DIY recipe index"><NumberInput value={v.diy_recipe} max={65535} onChange={(x) => update({ ...v, diy_recipe: x })} /></Field>
                <Field label="DIY ends (h:m:s)">
                  <div className="row">{[0, 1, 2].map((k) => <NumberInput key={k} value={v.diy_end[k]} max={k ? 59 : 23} width={60} onChange={(x) => { const d = v.diy_end.slice(); d[k] = x; update({ ...v, diy_end: d }); }} />)}</div>
                </Field>
              </div>
              <FlagTable values={v.event_flags} defs={gd.flags_villager} height={420} onChange={(f) => update({ ...v, event_flags: f })} />
            </div>
          )}
        </div>
      </Card>
      {replacing && <ReplaceModal v={v} name={name(v)} onClose={() => setReplacing(false)} onDone={(l) => { setList(l); api.villagerHouses().then(setHouses); }} />}
    </div></div>
  );
}

function General({ v, update }: { v: Villager; update: (v: Villager, now?: boolean) => void }) {
  const gd = useGD();
  const { settings } = useStore();
  const def = gd.villagers.find((x) => x.internal === v.internal_name)?.phrase ?? "";
  return (
    <div className="grid cols-2" style={{ gap: 16 }}>
      <div className="col" style={{ gap: 12 }}>
        <Field label="Personality"><Select value={v.personality} onChange={(p) => update({ ...v, personality: p }, true)} options={enumOptions(gd, "VillagerPersonality")} /></Field>
        <Field label="Catchphrase" hint={def ? `Default: “${def}”` : undefined}>
          <div className="row">
            <TextInput value={v.catchphrase} maxLength={24} onChange={(c) => update({ ...v, catchphrase: c })} />
            <Button iconOnly icon={<RefreshCcw />} title="Restore default catchphrase" onClick={() => update({ ...v, catchphrase: def }, true)} />
          </div>
        </Field>
        <ToggleRow title="Moving out" desc="The villager leaves the island at the next day change." on={v.moving_out} onChange={(m) => update({ ...v, moving_out: m }, true)} />
      </div>
      <div className="col" style={{ gap: 12 }}>
        {settings?.advanced && (
          <Field label="Species & variant (identity only)" hint="Changes who this villager is without updating their house or belongings. Prefer “Replace villager…”.">
            <VillagerPicker value={v.internal_name} onChange={(n) => {
              const sp = (gd.enums.VillagerSpecies ?? []).find(([k]) => k === n.slice(0, 3));
              if (sp) update({ ...v, species: sp[1], variant: Number(n.slice(3)), internal_name: n }, true);
            }} />
          </Field>
        )}
        <Callout kind="info" title="Tip">Use <b>Replace villager…</b> to swap in any villager with their own house, furniture and clothing. Their relationships with your residents are kept.</Callout>
      </div>
    </div>
  );
}

function Memories({ v, update }: { v: Villager; update: (v: Villager, now?: boolean) => void }) {
  const mems = v.memories.map((m, i) => ({ m, i })).filter(({ m }) => m.player.player_name);
  const [sel, setSel] = useState(mems[0]?.i ?? 0);
  useEffect(() => { if (!mems.find((x) => x.i === sel)) setSel(mems[0]?.i ?? 0); }, [v.index]);
  if (!mems.length) return <div className="faint">This villager doesn't remember any player yet.</div>;
  const m = v.memories[sel];
  const set = (patch: Partial<typeof m>) => { const memories = v.memories.slice(); memories[sel] = { ...m, ...patch }; update({ ...v, memories }); };
  return (
    <div className="grid" style={{ gridTemplateColumns: "220px 1fr", gap: 16, alignItems: "start" }}>
      <div className="list">
        {mems.map(({ m: x, i }) => (
          <div key={i} className={"list-item " + (i === sel ? "sel" : "")} onClick={() => setSel(i)}>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div className="bold ellipsis">{x.player.player_name}</div>
              <div className="tiny faint ellipsis">{x.player.town_name}</div>
              <div className="heart-bar" style={{ marginTop: 4 }}><div style={{ width: `${(x.friendship / 255) * 100}%` }} /></div>
            </div>
          </div>
        ))}
      </div>
      <div className="col" style={{ gap: 14 }}>
        <div className="grid cols-2" style={{ gap: 12 }}>
          <Field label={<><Heart size={13} /> Friendship ({m.friendship}/255)</>}>
            <div className="row">
              <input type="range" min={0} max={255} value={m.friendship} onChange={(e) => set({ friendship: Number(e.target.value) })} style={{ flex: 1, accentColor: "#ea5f86" }} />
              <Button size="sm" onClick={() => set({ friendship: 255 })}>Max</Button>
            </div>
          </Field>
          <Field label="Nickname for this player"><TextInput value={m.nickname} maxLength={10} onChange={(n) => set({ nickname: n })} /></Field>
        </div>
        <div className="label">Greetings</div>
        <div className="small faint" style={{ marginTop: -8 }}>What this villager says when meeting {m.player.player_name}. Stored with up to 47 characters; the speech bubble usually fits about 24.</div>
        <div className="grid cols-2" style={{ gap: 10 }}>
          {m.greetings.map((g, i) => (
            <CountedInput key={i} max={47} soft={24} value={g} placeholder={`Greeting ${i + 1}`} onChange={(t) => { const greetings = m.greetings.slice(); greetings[i] = t; set({ greetings }); }} />
          ))}
        </div>
      </div>
    </div>
  );
}

function SurfaceEditor({ value, onChange }: { value: RoomFloorWall; onChange: (r: RoomFloorWall) => void }) {
  return (
    <div className="grid cols-3" style={{ gap: 14 }}>
      {(["wall", "floor", "accent_wall"] as const).map((k) => (
        <div key={k} className="col" style={{ gap: 6 }}>
          <div className="label">{k === "wall" ? "Wallpaper" : k === "floor" ? "Flooring" : "Accent wall"}</div>
          <ItemEditor item={value[k]} allowWrap={false} compact onChange={(it) => onChange({ ...value, [k]: it })} />
        </div>
      ))}
    </div>
  );
}

function HomeTab({ v, house, updHouse, update }: { v: Villager; house: VillagerHouse; updHouse: (h: VillagerHouse) => void; update: (v: Villager, now?: boolean) => void }) {
  const gd = useGD();
  const { toast, touch } = useStore();
  const e = house.exterior;
  const setE = (patch: Partial<typeof e>) => updHouse({ ...house, exterior: { ...e, ...patch } });
  return (
    <div className="col" style={{ gap: 16 }}>
      <div className="row">
        <div className="label" style={{ flex: 1 }}><DoorOpen size={14} /> Exterior (house #{house.index + 1})</div>
        <Button size="sm" icon={<Download />} onClick={async () => { const p = await saveFile("Export house", `house-${house.index + 1}.nhvh2`, [F.nhvh]); if (p) { await api.exportVillagerHouse(house.index, p); toast("ok", "House exported"); } }}>Export</Button>
        <Button size="sm" icon={<Upload />} onClick={async () => { const p = await pickFile("Import house", [F.nhvh, F.all]); if (p) { try { await api.importVillagerHouse(house.index, p); touch(); toast("ok", "House imported"); } catch (er) { toast("error", "Import failed", String(er)); } } }}>Import</Button>
      </div>
      <div className="grid cols-3" style={{ gap: 12 }}>
        <Field label="Walls"><Select value={e.wall} onChange={(x) => setE({ wall: x, order_wall: x })} options={enumOptions(gd, "WallType")} /></Field>
        <Field label="Roof"><Select value={e.roof} onChange={(x) => setE({ roof: x, order_roof: x })} options={enumOptions(gd, "RoofType")} /></Field>
        <Field label="Door"><Select value={e.door} onChange={(x) => setE({ door: x, order_door: x })} options={enumOptions(gd, "DoorKind")} /></Field>
      </div>
      <div className="grid cols-2" style={{ gap: 12 }}>
        <Field label="House level"><NumberInput value={e.level} onChange={(x) => setE({ level: x })} /></Field>
        <div className="col" style={{ gap: 6 }}><div className="label">Door decoration</div><ItemEditor item={e.door_deco} compact allowWrap={false} onChange={(it) => setE({ door_deco: it })} /></div>
      </div>
      <div className="divider" />
      <div className="label">Interior (villager's own room settings)</div>
      <SurfaceEditor value={v.room} onChange={(room) => update({ ...v, room }, true)} />
      <div className="small faint">Furniture placement inside villager houses is managed by the game when the villager moves in.</div>
    </div>
  );
}

function ReplaceModal({ v, name, onClose, onDone }: { v: Villager; name: string; onClose: () => void; onDone: (l: Villager[]) => void }) {
  const { touch, toast } = useStore();
  const gd = useGD();
  const [pick, setPick] = useState(gd.villagers.find((x) => x.has_template)?.internal ?? "cat00");
  const [moveOut, setMoveOut] = useState(false);
  return (
    <Modal title={`Replace ${name}`} onClose={onClose} footer={<>
      <Button onClick={onClose}>Cancel</Button>
      <Button variant="primary" onClick={async () => {
        try { const l = await api.replaceVillager(v.index, pick, moveOut); onDone(l); touch(); toast("ok", `${gd.villagers.find((x) => x.internal === pick)?.name} moved in`); onClose(); }
        catch (e) { toast("error", "Replacement failed", String(e)); }
      }}>Replace</Button>
    </>}>
      <div className="col" style={{ gap: 14 }}>
        <div className="row" style={{ gap: 16 }}>
          <img src={villagerSprite(pick)} className="portrait lg" alt="" />
          <div className="col" style={{ flex: 1 }}>
            <Field label="New villager"><VillagerPicker value={pick} onlyTemplates onChange={setPick} /></Field>
          </div>
        </div>
        <ToggleRow title="Prepare to move out" desc="Flag the new villager as moving out right away (handy for adopting them on another island)." on={moveOut} onChange={setMoveOut} />
        <Callout kind="info">The new villager arrives with their default house, furniture and clothing. Your residents stay acquainted, starting from a fresh friendship level.</Callout>
      </div>
    </Modal>
  );
}
