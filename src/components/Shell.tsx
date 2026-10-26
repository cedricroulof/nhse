import {
  Award, Backpack, BookOpen, CalendarDays, ChevronDown, Flag, FolderOpen, Hammer, History, House, Landmark, LayoutDashboard, LogOut, Mail,
  Map as MapIcon, Package, Palette, Recycle, RotateCcw, Save, Settings, Smile, StickyNote, TrendingUp, User, Users, Wrench, X,
} from "lucide-react";
import React from "react";
import logo from "../logo.png";
import { useStore, type Route } from "../lib/store";
import { Badge, Button, Modal, Popover } from "./ui";

type NavDef = { route: Route; label: string; icon: React.ReactNode; advanced?: boolean };
const ISLAND: NavDef[] = [
  { route: "overview", label: "Overview", icon: <LayoutDashboard /> },
  { route: "villagers", label: "Villagers", icon: <Users /> },
  { route: "map", label: "Map", icon: <MapIcon /> },
  { route: "houses", label: "Houses", icon: <House /> },
  { route: "museum", label: "Museum", icon: <Landmark /> },
  { route: "designs", label: "Designs", icon: <Palette /> },
  { route: "turnips", label: "Turnips", icon: <TrendingUp /> },
  { route: "visitors", label: "Visitors", icon: <CalendarDays /> },
  { route: "bulletin", label: "Bulletin board", icon: <StickyNote /> },
  { route: "recycle", label: "Recycle bin", icon: <Recycle /> },
  { route: "landflags", label: "Island flags", icon: <Flag />, advanced: true },
];
const PLAYER: NavDef[] = [
  { route: "profile", label: "Profile", icon: <User /> },
  { route: "pockets", label: "Pockets", icon: <Backpack /> },
  { route: "storage", label: "Storage", icon: <Package /> },
  { route: "recipes", label: "DIY recipes", icon: <Hammer /> },
  { route: "catalog", label: "Catalog", icon: <BookOpen /> },
  { route: "reactions", label: "Reactions", icon: <Smile /> },
  { route: "achievements", label: "Nook Miles", icon: <Award /> },
  { route: "mail", label: "Mailbox", icon: <Mail /> },
  { route: "playerflags", label: "Player flags", icon: <Flag />, advanced: true },
];
const APP: NavDef[] = [
  { route: "tools", label: "Tools", icon: <Wrench /> },
  { route: "backups", label: "Backups", icon: <History /> },
  { route: "settings", label: "Settings", icon: <Settings /> },
];

export const TITLES: Record<Route, string> = Object.fromEntries([...ISLAND, ...PLAYER, ...APP].map((n) => [n.route, n.label])) as Record<Route, string>;

export function Sidebar() {
  const { save, route, go, player, setPlayer, settings, dirty } = useStore();
  const [pickerAt, setPickerAt] = React.useState<HTMLElement | null>(null);
  if (!save) return null;
  const nav = (items: NavDef[]) => items.filter((n) => !n.advanced || settings?.advanced).map((n) => (
    <button key={n.route} className={"nav-item " + (route === n.route ? "active" : "")} onClick={() => go(n.route)}>
      {n.icon}{n.label}
    </button>
  ));
  return (
    <aside className="sidebar">
      <div className="sidebar-top drag" />
      <div className="sidebar-scroll">
        <div className="island-card">
          <img src={logo} className="logo" alt="" />
          <div style={{ minWidth: 0 }}>
            <div className="name">{save.town_name || "Island"}</div>
            <div className="sub">v{save.version}{dirty && <> · <span style={{ color: "var(--warn)" }}>unsaved</span></>}</div>
          </div>
        </div>
        <div className="nav-section">
          <div className="nav-label">Island</div>
          {nav(ISLAND)}
        </div>
        <div className="nav-section">
          <div className="nav-label">
            <span>Resident</span>
          </div>
          <button className="nav-item" onClick={(e) => setPickerAt(e.currentTarget)} style={{ background: "var(--card)", border: "1px solid var(--border)", marginBottom: 4 }}>
            <User />
            <span className="ellipsis" style={{ flex: 1, color: "var(--text)" }}>{save.players[player]?.name || `Player ${player + 1}`}</span>
            <ChevronDown />
          </button>
          {pickerAt && (
            <Popover anchor={pickerAt} onClose={() => setPickerAt(null)} items={save.players.map((p) => ({
              label: `${p.name || "(unnamed)"}  ·  ${p.dir}`, icon: <User />, onClick: () => setPlayer(p.index),
            }))} />
          )}
          {nav(PLAYER)}
        </div>
        <div className="nav-section">
          <div className="nav-label">App</div>
          {nav(APP)}
        </div>
      </div>
    </aside>
  );
}

export function Topbar({ title, subtitle, actions }: { title: string; subtitle?: React.ReactNode; actions?: React.ReactNode }) {
  const { save, dirty, persist, close, confirm, revert, busy } = useStore();
  return (
    <header className="topbar drag">
      <div className="no-drag" style={{ minWidth: 0 }}>
        <h1>{title}</h1>
      </div>
      {subtitle && <div className="subtitle no-drag">{subtitle}</div>}
      <div className="spacer" />
      <div className="row no-drag">
        {actions}
        {save && (
          <>
            {dirty && <Badge kind="warn"><span className="dot" /> Unsaved changes</Badge>}
            <Button size="sm" variant="ghost" icon={<RotateCcw />} disabled={!dirty || !!busy} title="Discard unsaved changes" onClick={async () => {
              if (await confirm({ title: "Discard changes?", message: "Reload the save from disk and lose every unsaved edit.", danger: true, confirm: "Discard" })) revert();
            }}>Revert</Button>
            <Button size="sm" icon={<LogOut />} variant="ghost" title="Close save" onClick={async () => {
              if (!dirty || (await confirm({ title: "Close without saving?", message: "Your unsaved changes will be lost.", danger: true, confirm: "Close" }))) close();
            }} />
            <Button variant="primary" icon={<Save />} disabled={!!busy} onClick={persist} title="Save (⌘S)">Save</Button>
          </>
        )}
      </div>
    </header>
  );
}

export function ConfirmHost() {
  const { pending } = useStore();
  if (!pending) return null;
  return (
    <Modal title={pending.title} onClose={() => pending.resolve(false)} footer={<>
      <Button onClick={() => pending.resolve(false)}>{pending.cancel ?? "Cancel"}</Button>
      <Button variant={pending.danger ? "danger" : "primary"} autoFocus onClick={() => pending.resolve(true)}>{pending.confirm ?? "OK"}</Button>
    </>}>
      {pending.message && <div className="muted" style={{ fontSize: 13.5 }}>{pending.message}</div>}
    </Modal>
  );
}

export function Toasts() {
  const { toasts, dismiss } = useStore();
  return (
    <div className="toasts">
      {toasts.map((t) => (
        <div key={t.id} className={"toast " + t.kind}>
          {t.kind === "ok" ? <Save /> : t.kind === "error" ? <X /> : <FolderOpen />}
          <div style={{ flex: 1, minWidth: 0 }}>
            <div className="t">{t.title}</div>
            {t.detail && <div className="d">{t.detail}</div>}
          </div>
          <button className="btn ghost icon sm" onClick={() => dismiss(t.id)}><X /></button>
        </div>
      ))}
    </div>
  );
}

export function BusyOverlay() {
  const { busy } = useStore();
  if (!busy) return null;
  return (
    <div className="backdrop" style={{ background: "rgba(10,15,12,0.18)" }}>
      <div className="card card-pad row" style={{ gap: 12 }}><div className="spinner" /> <span className="bold">{busy}</span></div>
    </div>
  );
}
