import { ItemGridEditor } from "../components/ItemGridEditor";
import { Loading } from "../components/ui";
import { api } from "../lib/api";
import { useLoad, useStore } from "../lib/store";

export default function Pockets() {
  const { player, touch, save } = useStore();
  const [items, setItems] = useLoad(() => api.playerItems(player, "pocket"), [player]);
  const [profile] = useLoad(() => api.playerProfile(player), [player]);
  if (!items) return <Loading />;
  const unlocked = profile ? (profile.pocket_count < 20 ? profile.pocket_count : 20 + Math.min(20, profile.bag_count)) : undefined;
  return (
    <div className="content">
      <div className="page wide">
        <ItemGridEditor title="Pockets" items={items} columns={10} unlocked={unlocked} fileName={`${save?.players[player]?.name ?? "player"}-pockets`}
          onChange={(next) => { setItems(next); api.setPlayerItems(player, "pocket", next).then(touch); }} />
      </div>
    </div>
  );
}
