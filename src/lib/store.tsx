import React, { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from "react";
import { api, type AppInfo, type GameData, type SaveSummary, type Settings } from "./api";
import { buildIndex, type SearchEntry } from "./items";

export type Toast = { id: number; kind: "ok" | "error" | "warn" | "info"; title: string; detail?: string };
type ConfirmOpts = { title: string; message?: React.ReactNode; confirm?: string; danger?: boolean; cancel?: string };
type PendingConfirm = ConfirmOpts & { resolve: (v: boolean) => void };

export type Route =
  | "overview" | "villagers" | "map" | "museum" | "designs" | "turnips" | "visitors" | "bulletin" | "recycle" | "landflags" | "houses"
  | "profile" | "pockets" | "storage" | "recipes" | "catalog" | "reactions" | "achievements" | "mail" | "playerflags"
  | "backups" | "settings" | "tools";

interface Store {
  info: AppInfo | null;
  settings: Settings | null;
  updateSettings: (patch: Partial<Settings>) => Promise<void>;
  save: SaveSummary | null;
  gd: GameData | null;
  index: SearchEntry[];
  indexAll: SearchEntry[];
  player: number;
  setPlayer: (p: number) => void;
  route: Route;
  go: (r: Route) => void;
  dirty: boolean;
  touch: () => void;
  open: (path: string) => Promise<boolean>;
  close: () => Promise<void>;
  persist: () => Promise<void>;
  revert: () => Promise<void>;
  refreshSave: () => Promise<void>;
  rev: number;
  toast: (kind: Toast["kind"], title: string, detail?: string) => void;
  toasts: Toast[];
  dismiss: (id: number) => void;
  confirm: (o: ConfirmOpts) => Promise<boolean>;
  pending: PendingConfirm | null;
  run: <T>(fn: () => Promise<T>, success?: string) => Promise<T | undefined>;
  busy: string | null;
  setBusy: (s: string | null) => void;
}

const Ctx = createContext<Store | null>(null);
export const useStore = () => {
  const s = useContext(Ctx);
  if (!s) throw new Error("store missing");
  return s;
};
/** Game data is always loaded once a save is open. */
export const useGD = () => useStore().gd!;

export function StoreProvider({ children }: { children: React.ReactNode }) {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [save, setSave] = useState<SaveSummary | null>(null);
  const [gd, setGd] = useState<GameData | null>(null);
  const [player, setPlayer] = useState(0);
  const [route, setRoute] = useState<Route>("overview");
  const [dirty, setDirty] = useState(false);
  const [rev, setRev] = useState(0);
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [pending, setPending] = useState<PendingConfirm | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const tid = useRef(1);

  const toast = useCallback((kind: Toast["kind"], title: string, detail?: string) => {
    const id = tid.current++;
    setToasts((t) => [...t.slice(-4), { id, kind, title, detail }]);
    setTimeout(() => setToasts((t) => t.filter((x) => x.id !== id)), kind === "error" ? 9000 : 4200);
  }, []);
  const dismiss = useCallback((id: number) => setToasts((t) => t.filter((x) => x.id !== id)), []);

  const run = useCallback(async <T,>(fn: () => Promise<T>, success?: string) => {
    try {
      const r = await fn();
      if (success) toast("ok", success);
      return r;
    } catch (e) {
      toast("error", "Something went wrong", String(e));
      return undefined;
    }
  }, [toast]);

  // theme
  useEffect(() => {
    const t = settings?.theme ?? "system";
    if (t === "system") delete document.documentElement.dataset.theme;
    else document.documentElement.dataset.theme = t;
  }, [settings?.theme]);

  useEffect(() => {
    api.appInfo().then((i) => { setInfo(i); setSettings(i.settings); }).catch((e) => toast("error", "Failed to start", String(e)));
  }, [toast]);

  // game data follows the language setting
  useEffect(() => {
    if (!settings) return;
    api.gameData(settings.language).then(setGd).catch((e) => toast("error", "Failed to load game data", String(e)));
  }, [settings?.language, toast]);

  const index = useMemo(() => (gd ? buildIndex(gd) : []), [gd]);
  const indexAll = useMemo(() => (gd ? buildIndex(gd, true) : []), [gd]);

  const updateSettings = useCallback(async (patch: Partial<Settings>) => {
    if (!settings) return;
    const next = await api.setSettings({ ...settings, ...patch });
    setSettings(next);
  }, [settings]);

  const open = useCallback(async (path: string) => {
    setBusy("Opening save…");
    try {
      const s = await api.openSave(path);
      setSave(s);
      setDirty(false);
      setPlayer(0);
      setRev((r) => r + 1);
      setRoute("overview");
      const i = await api.appInfo();
      setInfo(i);
      setSettings(i.settings);
      if (s.backup) toast("ok", `Opened ${s.town_name}`, "A backup of the save files was stored automatically.");
      else toast("ok", `Opened ${s.town_name}`);
      return true;
    } catch (e) {
      toast("error", "Could not open this save", String(e));
      return false;
    } finally {
      setBusy(null);
    }
  }, [toast]);

  const close = useCallback(async () => {
    await api.closeSave();
    setSave(null);
    setDirty(false);
  }, []);

  const persist = useCallback(async () => {
    if (!save) return;
    setBusy("Saving…");
    try {
      const s = await api.saveChanges();
      setSave(s);
      setDirty(false);
      toast("ok", "Saved", s.is_zip ? "The ZIP archive was updated." : "All files were re-hashed, encrypted and written.");
    } catch (e) {
      toast("error", "Save failed", String(e));
    } finally {
      setBusy(null);
    }
  }, [save, toast]);

  const revert = useCallback(async () => {
    const s = await api.revertSave();
    setSave(s);
    setDirty(false);
    setRev((r) => r + 1);
  }, []);

  const refreshSave = useCallback(async () => {
    const s = await api.saveSummary();
    setSave(s);
  }, []);

  const touch = useCallback(() => setDirty(true), []);

  const confirm = useCallback((o: ConfirmOpts) => new Promise<boolean>((resolve) => setPending({ ...o, resolve })), []);

  const go = useCallback((r: Route) => setRoute(r), []);

  const value: Store = {
    info, settings, updateSettings, save, gd, index, indexAll, player, setPlayer, route, go, dirty, touch, open, close, persist,
    revert, refreshSave, rev, toast, toasts, dismiss, confirm, pending: pending
      ? { ...pending, resolve: (v: boolean) => { pending.resolve(v); setPending(null); } }
      : null,
    run, busy, setBusy,
  };
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

/** Loads data with `fn` whenever deps (or the save revision) change. */
export function useLoad<T>(fn: () => Promise<T>, deps: unknown[]): [T | null, (v: T | null) => void, () => void, boolean] {
  const { rev, toast } = useStore();
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [tick, setTick] = useState(0);
  useEffect(() => {
    let live = true;
    setLoading(true);
    fn().then((d) => { if (live) { setData(d); setLoading(false); } })
      .catch((e) => { if (live) { setLoading(false); toast("error", "Failed to load", String(e)); } });
    return () => { live = false; };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [...deps, rev, tick]);
  return [data, setData, () => setTick((t) => t + 1), loading];
}
