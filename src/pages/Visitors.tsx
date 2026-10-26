import { Card, Check, Field, Loading, NumberInput, Select } from "../components/ui";
import { enumOptions } from "../components/Widgets";
import { api, type Visitors as V } from "../lib/api";
import { useGD, useLoad, useStore } from "../lib/store";

const DAYS = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

export default function Visitors() {
  const { touch } = useStore();
  const gd = useGD();
  const [v, setV] = useLoad(api.visitors, []);
  if (!v) return <Loading />;
  const upd = (next: V) => { setV(next); api.setVisitors(next).then(touch); };
  const opts = enumOptions(gd, "VisitorNPC");
  return (
    <div className="content">
      <div className="page">
        <Card title="Weekly visitor schedule" desc="Special characters visiting the plaza this week">
          <table className="table">
            <thead><tr><th>Day</th><th>Visitor</th><th>Planned visitor</th><th>K.K. concert</th><th>Birthday concert</th></tr></thead>
            <tbody>
              {DAYS.map((d, i) => (
                <tr key={d}>
                  <td className="bold">{d}</td>
                  <td><Select value={v.schedule[i]} options={opts} onChange={(x) => { const s = v.schedule.slice(); s[i] = x; upd({ ...v, schedule: s }); }} /></td>
                  <td><Select value={v.init_schedule[i]} options={opts} onChange={(x) => { const s = v.init_schedule.slice(); s[i] = x; upd({ ...v, init_schedule: s }); }} /></td>
                  <td><Check checked={v.is_normal_live[i]} onChange={(x) => { const s = v.is_normal_live.slice(); s[i] = x; upd({ ...v, is_normal_live: s }); }} /></td>
                  <td><Check checked={v.is_birthday_live[i]} onChange={(x) => { const s = v.is_birthday_live.slice(); s[i] = x; upd({ ...v, is_birthday_live: s }); }} /></td>
                </tr>
              ))}
            </tbody>
          </table>
        </Card>
        <Card title="Nightly visitors">
          <div className="grid cols-2" style={{ gap: 12 }}>
            <Field label="Wisp day"><NumberInput value={v.day_wisp} min={-2147483648} max={2147483647} onChange={(x) => upd({ ...v, day_wisp: x })} /></Field>
            <Field label="Celeste day"><NumberInput value={v.day_celeste} min={-2147483648} max={2147483647} onChange={(x) => upd({ ...v, day_celeste: x })} /></Field>
          </div>
        </Card>
      </div>
    </div>
  );
}
