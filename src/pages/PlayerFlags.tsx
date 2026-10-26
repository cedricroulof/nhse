import { FlagTable } from "../components/FlagTable";
import { Callout, Card, Loading } from "../components/ui";
import { api } from "../lib/api";
import { useGD, useLoad, useStore } from "../lib/store";

export default function PlayerFlags() {
  const { touch, player } = useStore();
  const gd = useGD();
  const [f, setF] = useLoad(() => api.playerFlags(player), [player]);
  if (!f) return <Loading />;
  return (
    <div className="content"><div className="page">
      <Callout kind="warn" title="Advanced">Player event flags track tutorials and story progress for this resident. Keep a backup before editing.</Callout>
      <Card title="Player event flags"><FlagTable values={f} defs={gd.flags_player} onChange={(v) => { setF(v); api.setPlayerFlags(player, v).then(touch); }} /></Card>
    </div></div>
  );
}
