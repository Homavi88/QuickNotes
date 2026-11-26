import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { MainWindow } from "./components/main/MainWindow";
import { QuickWindow } from "./components/quick/QuickWindow";

export default function App() {
  const [label, setLabel] = useState<string | null>(null);

  useEffect(() => {
    try {
      setLabel(getCurrentWindow().label);
    } catch {
      setLabel("main");
    }
  }, []);

  if (!label) return null;
  if (label === "quick") return <QuickWindow />;
  return <MainWindow />;
}
