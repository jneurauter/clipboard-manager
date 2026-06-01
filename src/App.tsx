import { useEffect, useState } from "react";
import { Copy } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ClipboardImage } from "./ClipboardImage";
import { TextHistoryItem } from "./TextHistoryItem";
import "./App.css";

type ClipboardItem =
  | { kind: "text"; content: string }
  | { kind: "image"; data_url: string };

export default function App() {
  const [history, setHistory] = useState<ClipboardItem[]>([]);

  const refreshHistory = async () => {
    const newHistory = await invoke<ClipboardItem[]>("get_history");
    setHistory(newHistory);
  };

  const handleCopy = async (item: ClipboardItem) => {
    await invoke("copy_to_clipboard", { item });
  };

  const handleClear = async () => {
    await invoke("clear_history");
  };

  useEffect(() => {
    refreshHistory();

    const unlisten = listen<ClipboardItem[]>("clipboard_update", (event) => {
      setHistory(event.payload);
      // Keep history up to date when the clipboard changes
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  return (
    <div className="glass-card">
        <header className="glass-card-header">
          <div className="header-title-group">
            <div>
              <h1>Clipboard Manager</h1>
              <p className="subtitle">
                {history.length} item{history.length === 1 ? "" : "s"} in history
              </p>
            </div>
          </div>
          <button
            type="button"
            className="btn-clear"
            onClick={handleClear}
            disabled={history.length === 0}
            title="Clear history"
          >
            Clear History
          </button>
        </header>

        <ul className="history-list">
          {history.length === 0 ? (
            <li className="empty-state">
              <p>No clipboard history</p>
            </li>
          ) : (
            history.map((item, i) =>
              item.kind === "text" ? (
                <TextHistoryItem
                  key={i}
                  content={item.content}
                  onCopy={() => handleCopy(item)}
                />
              ) : (
                <li key={i} className="item-card item-card-image">
                  <button
                    type="button"
                    className="btn-copy btn-copy-left"
                    onClick={() => handleCopy(item)}
                    aria-label="Copy item"
                  >
                    <Copy className="copy-icon" aria-hidden="true" />
                  </button>
                  <ClipboardImage
                    dataUrl={item.data_url}
                    listLength={history.length}
                  />
                </li>
              ),
            )
          )}
        </ul>
      </div>
  );
}
