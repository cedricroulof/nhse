import { FlagTable } from "../components/FlagTable";
import { Callout, Card, Loading } from "../components/ui";
import { api } from "../lib/api";
import { useGD, useLoad, useStore } from "../lib/store";

export default function LandFlags() {
  const { touch } = useStore();
  const gd = useGD();
  const [f, setF] = useLoad(api.landFlags, []);
  if (!f) return <Loading />;
  return (
    <div className="content"><div className="page">
      <Callout kind="warn" title="Advanced">Island event flags drive story progress. Changing them can break events — keep a backup.</Callout>
      <Card title="Island event flags"><FlagTable values={f} defs={gd.flags_land} onChange={(v) => { setF(v); api.setLandFlags(v).then(touch); }} /></Card>
    </div></div>
  );
}
