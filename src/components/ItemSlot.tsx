import React from "react";
import type { Item } from "../lib/api";
import { count, displayId, itemColor, itemLabel, itemSprite, NONE_ID, variantOf } from "../lib/items";
import { useStore } from "../lib/store";

const WRAP_COLORS = ["#f5d24a", "#f29ac4", "#f39a4c", "#a8d86a", "#46a85b", "#8ad8c4", "#8cc8f0", "#a77fd6", "#3b4b8f", "#3d7fd6", "#f2f2f2", "#d94a4a", "#d8b14a", "#8a5a3a", "#9a9a9a", "#2a2a2a"];

export function ItemSlot({ item, selected, multi, locked, onMouseDown, onDoubleClick, onContextMenu, size, title, draggable, onDragStart, onDragOver, onDrop, drop, label }: {
  item: Item; selected?: boolean; multi?: boolean; locked?: boolean; size?: number; title?: string; label?: string; drop?: boolean;
  onMouseDown?: (e: React.MouseEvent) => void; onDoubleClick?: () => void; onContextMenu?: (e: React.MouseEvent) => void;
  draggable?: boolean; onDragStart?: (e: React.DragEvent) => void; onDragOver?: (e: React.DragEvent) => void; onDrop?: (e: React.DragEvent) => void;
}) {
  const { gd } = useStore();
  const empty = item.id === NONE_ID;
  const id = displayId(item);
  const stack = gd && id < gd.kinds.length ? count(item) : 0;
  const wrapped = !empty && item.additional !== 0 && (item.additional & 3) !== 0;
  const name = gd ? itemLabel(gd, item) : "";
  const showCount = !empty && stack > 0 && gd && isStackable(gd, id);
  return (
    <div
      className={["slot", empty && "vacant", selected && "sel", multi && "multi", locked && "locked", drop && "drop"].filter(Boolean).join(" ")}
      style={size ? ({ "--slot": `${size}px` } as React.CSSProperties) : undefined}
      title={title ?? (empty ? "Empty" : name)}
      onMouseDown={onMouseDown}
      onDoubleClick={onDoubleClick}
      onContextMenu={onContextMenu}
      draggable={draggable && !empty}
      onDragStart={onDragStart}
      onDragOver={onDragOver}
      onDrop={onDrop}
    >
      {!empty && (
        <img
          src={itemSprite(id, variantOf(item))}
          alt=""
          loading="lazy"
          onError={(e) => { (e.target as HTMLImageElement).style.display = "none"; ((e.target as HTMLImageElement).parentElement!).style.background = gd ? itemColor(gd, id) : ""; }}
        />
      )}
      {showCount && <span className="count">{stack + 1}</span>}
      {wrapped && <span className="wrap-dot" style={{ background: (item.additional & 3) === 1 ? WRAP_COLORS[(item.additional >> 2) & 15] : "#c9a46a" }} />}
      {label && <span className="corner">{label}</span>}
    </div>
  );
}

const stackCache = new Map<number, boolean>();
/** Whether an item stacks (quantity is stored as count-1). */
export function isStackable(gd: { kind_names: string[]; kinds: number[] }, id: number): boolean {
  if (stackCache.has(id)) return stackCache.get(id)!;
  const kn = gd.kind_names[gd.kinds[id]] ?? "";
  const r = STACKABLE.has(kn) || kn === "Kind_Flower" || kn.startsWith("UnitIcon_Flw");
  stackCache.set(id, r);
  return r;
}

export const STACK_MAX: Record<string, number> = {
  Kind_CookingMaterial: 50, Kind_CraftRemake: 50, Kind_Fence: 50, Kind_WeedLight: 50, Kind_Weed: 99,
  Kind_CraftMaterial: 30, Kind_Ore: 30, Kind_EasterEgg: 30, Kind_LoveCrystal: 30, Kind_Candy: 30, Kind_Vine: 30,
  Kind_YutaroWisp: 5, Kind_RollanTicket: 5, Kind_Feather: 3,
};
for (const k of ["Kind_PartyPopper", "Kind_BlowBubble", "Kind_FierworkHand", "Kind_TreeSeedling", "Kind_BushSeedling", "Kind_VegeSeedling",
  "Kind_Vegetable", "Kind_FlowerSeed", "Kind_Flower", "Kind_Fruit", "Kind_Mushroom", "Kind_Turnip", "Kind_FishBait", "Kind_PitFallSeed",
  "Kind_Medicine", "Kind_Honeycomb", "Kind_SnowCrystal", "Kind_AutumnLeaf", "Kind_Sakurapetal", "Kind_XmasDeco", "Kind_StarPiece",
  "Kind_ShellDrift", "Kind_WrappingPaper", "Kind_Otoshidama", "Kind_Money", "Kind_BdayCupcake", "Kind_JohnnyQuest", "Kind_JohnnyQuestDust",
  "Kind_PirateQuest", "Kind_TailorTicket", "Kind_MilePlaneTicket", "Kind_SouvenirChocolate"]) STACK_MAX[k] = 10;
const STACKABLE = new Set(Object.keys(STACK_MAX));
