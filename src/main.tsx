import React from "react";
import ReactDOM from "react-dom/client";
import "@fontsource-variable/inter";
import "./styles/theme.css";
import "./styles/app.css";
import App from "./App";
import { StoreProvider } from "./lib/store";

// block the default browser context menu (custom menus are used where useful)
window.addEventListener("contextmenu", (e) => {
  const t = e.target as HTMLElement;
  if (!["INPUT", "TEXTAREA"].includes(t.tagName)) e.preventDefault();
});

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <StoreProvider>
      <App />
    </StoreProvider>
  </React.StrictMode>,
);
