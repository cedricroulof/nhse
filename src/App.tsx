import React, { Suspense, lazy, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { BusyOverlay, ConfirmHost, Sidebar, TITLES, Toasts, Topbar } from "./components/Shell";
import { Loading } from "./components/ui";
import { useStore, type Route } from "./lib/store";
import { Welcome } from "./pages/Welcome";

const pages: Record<Route, React.LazyExoticComponent<React.ComponentType>> = {
  overview: lazy(() => import("./pages/Overview")),
  villagers: lazy(() => import("./pages/Villagers")),
  map: lazy(() => import("./pages/MapEditor")),
  houses: lazy(() => import("./pages/Houses")),
  museum: lazy(() => import("./pages/Museum")),
  designs: lazy(() => import("./pages/Designs")),
  turnips: lazy(() => import("./pages/Turnips")),
  visitors: lazy(() => import("./pages/Visitors")),
  bulletin: lazy(() => import("./pages/Bulletin")),
  recycle: lazy(() => import("./pages/RecycleBin")),
  landflags: lazy(() => import("./pages/LandFlags")),
  profile: lazy(() => import("./pages/Profile")),
  pockets: lazy(() => import("./pages/Pockets")),
  storage: lazy(() => import("./pages/Storage")),
  recipes: lazy(() => import("./pages/Recipes")),
  catalog: lazy(() => import("./pages/Catalog")),
  reactions: lazy(() => import("./pages/Reactions")),
  achievements: lazy(() => import("./pages/Achievements")),
  mail: lazy(() => import("./pages/Mailbox")),
  playerflags: lazy(() => import("./pages/PlayerFlags")),
  tools: lazy(() => import("./pages/Tools")),
  backups: lazy(() => import("./pages/Backups")),
  settings: lazy(() => import("./pages/SettingsPage")),
};

export default function App() {
  const { save, route, persist, dirty, confirm, gd, player } = useStore();

  // ⌘S / Ctrl+S
  useEffect(() => {
    const k = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "s") { e.preventDefault(); if (save) persist(); }
    };
    window.addEventListener("keydown", k);
    return () => window.removeEventListener("keydown", k);
  }, [save, persist]);

  // warn about unsaved changes when closing the window
  useEffect(() => {
    const w = getCurrentWindow();
    const un = w.onCloseRequested(async (e) => {
      if (!dirty) return;
      e.preventDefault();
      if (await confirm({ title: "Quit without saving?", message: "You have unsaved changes. They will be lost.", danger: true, confirm: "Quit" })) await w.destroy();
    });
    return () => { un.then((f) => f()); };
  }, [dirty, confirm]);

  const Page = pages[route];
  const needsSave = !["settings", "backups"].includes(route);
  return (
    <div className="app">
      {save ? <Sidebar /> : null}
      <main className="main">
        {save || !needsSave ? (
          <>
            <Topbar title={TITLES[route]} subtitle={route !== "overview" && save ? save.town_name : undefined} />
            {gd ? (
              <Suspense fallback={<Loading />}>
                <Page key={`${route}:${player}`} />
              </Suspense>
            ) : <Loading />}
          </>
        ) : <Welcome />}
      </main>
      <ConfirmHost />
      <Toasts />
      <BusyOverlay />
    </div>
  );
}
