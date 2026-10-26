import React from "react";
import ReactDOM from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [v, setV] = React.useState("");
  React.useEffect(() => { invoke<string>("core_version").then(setV).catch(() => setV("?")); }, []);
  return <h1>NHSE {v}</h1>;
}

ReactDOM.createRoot(document.getElementById("root")!).render(<App />);
