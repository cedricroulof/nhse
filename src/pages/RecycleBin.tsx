import { ItemGridEditor } from "../components/ItemGridEditor";
import { Loading } from "../components/ui";
import { api } from "../lib/api";
import { useLoad, useStore } from "../lib/store";

export default function RecycleBin() {
  const { touch } = useStore();
  const [items, setItems] = useLoad(api.recycleBin, []);
  if (!items) return <Loading />;
  return (
    <div className="content">
      <div className="page wide">
        <ItemGridEditor title="Resident Services recycle bin" items={items} columns={10} fileName="recycle-bin"
          onChange={(next) => { setItems(next); api.setRecycleBin(next).then(touch); }} />
      </div>
    </div>
  );
}
