import { CloudSun, Dices, Landmark, Palette, Plane, Sprout, Tent, Users } from "lucide-react";
import logo from "../logo.png";
import { Button, Callout, Card, Field, NumberInput, Select, Stat } from "../components/ui";
import { DateInput, VillagerPicker, enumOptions, fruitOptions } from "../components/Widgets";
import { api, type IslandSettings } from "../lib/api";
import { villagerSprite } from "../lib/items";
import { useGD, useLoad, useStore } from "../lib/store";

export default function Overview() {
  const { save, touch, go, setPlayer } = useStore();
  const gd = useGD();
  const [s, setS] = useLoad(api.islandSettings, []);
  const [villagers] = useLoad(api.villagers, []);
  const [museum] = useLoad(api.museum, []);
  const [designs] = useLoad(() => api.designs("normal"), []);
  if (!save) return null;

  const upd = (patch: Partial<IslandSettings>) => {
    if (!s) return;
    const next = { ...s, ...patch };
    setS(next);
    api.setIslandSettings(next).then(touch);
  };
  const living = villagers?.filter((v) => v.species !== 35 && v.species !== 255) ?? [];
  const donated = museum?.items.filter((i) => i.id !== 0xfffe).length ?? 0;
  const campName = s ? `${gd.enums.VillagerSpecies?.find(([, v]) => v === s.camp_species)?.[0] ?? "non"}${String(s.camp_variant).padStart(2, "0")}` : "";

  return (
    <div className="content">
      <div className="page">
        {save.compatibility_mode && <Callout kind="warn" title="Compatibility mode">{save.warnings[0]}</Callout>}
        {save.warnings.filter((_w, i) => !(save.compatibility_mode && i === 0)).map((w) => <Callout key={w} kind="warn">{w}</Callout>)}

        <div className="card" style={{ overflow: "hidden" }}>
          <div style={{ padding: "22px 24px", display: "flex", gap: 18, alignItems: "center", background: "linear-gradient(120deg, var(--accent-soft), transparent 60%)" }}>
            <img src={logo} style={{ width: 56, height: 56, borderRadius: 14, boxShadow: "var(--shadow)" }} alt="" />
            <div style={{ flex: 1, minWidth: 0 }}>
              <h1 style={{ fontSize: 26 }}>{save.town_name}</h1>
              <div className="muted">Game version {save.version} · last saved {s?.last_saved ?? "…"}</div>
              <div className="faint tiny ellipsis" style={{ marginTop: 2 }}>{save.path}</div>
            </div>
          </div>
        </div>

        <div className="grid cols-4">
          <Stat label="Residents" icon={<Users />} value={save.players.length} />
          <Stat label="Villagers" icon={<Sprout />} value={`${living.length}/10`} />
          <Stat label="Museum" icon={<Landmark />} value={donated.toLocaleString()} />
          <Stat label="Designs" icon={<Palette />} value={`${designs?.filter((d) => !d.empty).length ?? 0}/${save.pattern_count}`} />
        </div>

        <div className="grid cols-2">
          <Card title="Residents">
            <div className="list">
              {save.players.map((p) => (
                <div key={p.index} className="list-item" onClick={() => { setPlayer(p.index); go("profile"); }}>
                  <div className="portrait sm" style={{ display: "grid", placeItems: "center", fontWeight: 700, color: "var(--accent)" }}>{(p.name || "?")[0]}</div>
                  <div style={{ flex: 1 }}><div className="bold">{p.name || "(unnamed)"}</div><div className="tiny faint">{p.dir}{p.index === 0 ? " · island representative" : ""}</div></div>
                </div>
              ))}
            </div>
          </Card>
          <Card title="Villagers" actions={<Button size="sm" onClick={() => go("villagers")}>Edit</Button>}>
            <div style={{ display: "grid", gridTemplateColumns: "repeat(5, 1fr)", gap: 8 }}>
              {living.map((v) => (
                <div key={v.index} className="col" style={{ alignItems: "center", gap: 4 }} title={v.internal_name}>
                  <img src={villagerSprite(v.internal_name)} className="portrait" alt="" />
                  <div className="tiny ellipsis" style={{ maxWidth: 80 }}>{gd.villagers.find((x) => x.internal === v.internal_name)?.name ?? v.internal_name}</div>
                </div>
              ))}
            </div>
          </Card>
        </div>

        {s && (
          <div className="grid cols-2">
            <Card title="Island" desc="Climate, airport and native species">
              <div className="grid cols-2" style={{ gap: 12 }}>
                <Field label="Hemisphere"><Select value={s.hemisphere} onChange={(v) => upd({ hemisphere: v })} options={enumOptions(gd, "Hemisphere")} /></Field>
                <Field label={<><Plane size={13} /> Airport color</>}><Select value={s.airport_color} onChange={(v) => upd({ airport_color: v })} options={enumOptions(gd, "AirportColor")} /></Field>
                <Field label="Native fruit"><Select value={s.specialty_fruit} onChange={(v) => upd({ specialty_fruit: v })} options={fruitOptions(gd)} /></Field>
                <Field label="Sister fruit"><Select value={s.sister_fruit} onChange={(v) => upd({ sister_fruit: v })} options={fruitOptions(gd)} /></Field>
                <Field label="Native flower"><Select value={s.specialty_flower} onChange={(v) => upd({ specialty_flower: v })} options={enumOptions(gd, "IslandFlowers")} /></Field>
                <Field label="Sister flower"><Select value={s.sister_flower} onChange={(v) => upd({ sister_flower: v })} options={enumOptions(gd, "IslandFlowers")} /></Field>
              </div>
            </Card>
            <Card title={<span className="row"><CloudSun size={16} /> Weather</span>} desc="The seed determines the weather pattern of every day">
              <div className="col" style={{ gap: 12 }}>
                <Field label="Weather seed">
                  <div className="row"><NumberInput value={s.weather_seed} onChange={(v) => upd({ weather_seed: v })} />
                    <Button iconOnly icon={<Dices />} title="Randomize" onClick={() => upd({ weather_seed: (Math.random() * 0x7fffffff) >>> 0 })} /></div>
                </Field>
                <div className="grid cols-2" style={{ gap: 12 }}>
                  <Field label="Tour hemisphere"><Select value={s.tour_hemisphere} onChange={(v) => upd({ tour_hemisphere: v })} options={enumOptions(gd, "Hemisphere")} /></Field>
                  <Field label="Tour weather seed"><NumberInput value={s.tour_weather_seed} onChange={(v) => upd({ tour_weather_seed: v })} /></Field>
                </div>
              </div>
            </Card>
            <Card title={<span className="row"><Tent size={16} /> Campsite</span>} desc="The villager currently staying at the campsite">
              <div className="grid cols-2" style={{ gap: 12 }}>
                <Field label="Visitor"><VillagerPicker value={campName} onChange={(n) => {
                  const sp = (gd.enums.VillagerSpecies ?? []).find(([k]) => k === n.slice(0, 3));
                  if (sp) upd({ camp_species: sp[1], camp_variant: Number(n.slice(3)) });
                }} /></Field>
                <Field label="Status"><NumberInput value={s.camp_status} max={255} onChange={(v) => upd({ camp_status: v })} /></Field>
                <Field label="Last visit"><DateInput value={s.camp_last_visit} onChange={(d) => upd({ camp_last_visit: d })} allowEmpty /></Field>
              </div>
            </Card>
          </div>
        )}
      </div>
    </div>
  );
}
