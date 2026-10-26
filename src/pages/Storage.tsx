import { ArrowDownAZ, PackagePlus, Rows3 } from "lucide-react";
import { useState } from "react";
import { ItemGridEditor } from "../components/ItemGridEditor";
import { Button, Check, Loading, MenuButton, Modal } from "../components/ui";
import { api, itemSet, NONE, type Item } from "../lib/api";
import { useGD, useLoad, useStore } from "../lib/store";

const SETS: [string, string][] = [
  ["wearables", "All wearables (clothing, hats, bags…)"], ["furniture", "All furniture"], ["interior", "Wallpaper, flooring & rugs"],
  ["tools", "Tools"], ["materials", "Crafting materials (full stacks)"], ["music", "K.K. songs"], ["fossils", "Fossils"], ["art", "Art"],
  ["recipes", "Every DIY recipe card"],
];

export default function Storage() {
  const { player, touch, save, toast } = useStore();
  const gd = useGD();
  const [items, setItems] = useLoad(() => api.playerItems(player, "storage"), [player]);
  const [profile] = useLoad(() => api.playerProfile(player), [player]);
  const [fill, setFill] = useState<string | null>(null);
  const [variants, setVariants] = useState(true);
  if (!items) return <Loading />;
  const commit = (next: Item[]) => { setItems(next); api.setPlayerItems(player, "storage", next).then(touch); };

  const doFill = async () => {
    if (!fill) return;
    const list = await itemSet(fill, variants);
    const have = new Set(items.filter((i) => i.id !== 0xfffe).map((i) => `${i.id}:${i.free}`));
    const todo = list.filter((i) => !have.has(`${i.id}:${i.free}`));
    const next = items.slice();
    let k = 0;
    for (let i = 0; i < next.length && k < todo.length; i++) if (next[i].id === 0xfffe) next[i] = todo[k++];
    commit(next);
    setFill(null);
    toast(k < todo.length ? "warn" : "ok", `Added ${k.toLocaleString()} items to storage`, k < todo.length ? `${todo.length - k} did not fit — storage is full.` : undefined);
  };
  const compact = () => {
    const filled = items.filter((i) => i.id !== 0xfffe);
    commit([...filled, ...Array(items.length - filled.length).fill(NONE)]);
  };
  const sort = () => {
    const filled = items.filter((i) => i.id !== 0xfffe).sort((a, b) => (gd.items[a.id] ?? "").localeCompare(gd.items[b.id] ?? "") || a.free - b.free);
    commit([...filled, ...Array(items.length - filled.length).fill(NONE)]);
  };

  return (
    <div className="content">
      <div className="page wide">
        <ItemGridEditor title="Home storage" items={items} columns={10} pageSize={200} unlocked={profile?.storage_count}
          fileName={`${save?.players[player]?.name ?? "player"}-storage`} onChange={commit}
          toolbar={<>
            <MenuButton size="sm" label="Fill" icon={<PackagePlus />} items={SETS.map(([k, l]) => ({ label: l, onClick: () => setFill(k) }))} />
            <Button size="sm" iconOnly icon={<Rows3 />} title="Remove gaps" onClick={compact} />
            <Button size="sm" iconOnly icon={<ArrowDownAZ />} title="Sort by name" onClick={sort} />
          </>} />
      </div>
      {fill && (
        <Modal title={SETS.find(([k]) => k === fill)?.[1] ?? "Fill"} onClose={() => setFill(null)} footer={<>
          <Button onClick={() => setFill(null)}>Cancel</Button><Button variant="primary" onClick={doFill}>Add to empty slots</Button></>}>
          <div className="col">
            <div className="muted small">Items you already have (same item and variation) are skipped. Only empty slots are used.</div>
            {!["fossils", "art", "recipes", "music", "materials"].includes(fill) && <Check label="Include every color variation" checked={variants} onChange={setVariants} />}
          </div>
        </Modal>
      )}
    </div>
  );
}
