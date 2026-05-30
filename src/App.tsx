import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ClipboardImage } from "./ClipboardImage";
import { TextHistoryItem } from "./TextHistoryItem";
import "./App.css";

type ClipboardItem =
  | { kind: "text"; content: string }
  | { kind: "image"; data_url: string };

function pruneExpanded(expanded: Set<number>, length: number): Set<number> {
  const next = new Set<number>();
  for (const index of expanded) {
    if (index < length) {
      next.add(index);
    }
  }
  return next.size === expanded.size ? expanded : next;
}

export default function App() {
  const [history, setHistory] = useState<ClipboardItem[]>([]);
  const [expandedText, setExpandedText] = useState<Set<number>>(new Set());

  const refreshHistory = async () => {
    const newHistory = await invoke<ClipboardItem[]>("get_history");
    setHistory(newHistory);
    setExpandedText((prev) => pruneExpanded(prev, newHistory.length));
  };

  const handleCopy = async (item: ClipboardItem) => {
    await invoke("copy_to_clipboard", { item });
    refreshHistory();
  };

  const handleClear = async () => {
    await invoke("clear_history");
    setExpandedText(new Set());
  };

  const toggleExpanded = (index: number) => {
    setExpandedText((prev) => {
      const next = new Set(prev);
      if (next.has(index)) {
        next.delete(index);
      } else {
        next.add(index);
      }
      return next;
    });
  };

  useEffect(() => {
    refreshHistory();

    const unlisten = listen<ClipboardItem[]>("clipboard_update", (event) => {
      setHistory(event.payload);
      setExpandedText((prev) => pruneExpanded(prev, event.payload.length));
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  return (
    <div className="app">
      <header className="app-header">
        <h1>Clipboard Manager</h1>
        <button
          type="button"
          className="btn-clear"
          onClick={handleClear}
          disabled={history.length === 0}
        >
          Clear history
        </button>
      </header>

      <ul className="history-list">
        {history.map((item, i) =>
          item.kind === "text" ? (
            <TextHistoryItem
              key={i}
              content={item.content}
              expanded={expandedText.has(i)}
              onToggleExpand={() => toggleExpanded(i)}
              onCopy={() => handleCopy(item)}
            />
          ) : (
            <li key={i} className="history-item">
              <ClipboardImage
                dataUrl={item.data_url}
                listLength={history.length}
              />
              <button
                type="button"
                className="btn-copy"
                onClick={() => handleCopy(item)}
              >
                Copy
              </button>
            </li>
          ),
        )}
      </ul>

      {history.length === 0 && (
        <p className="empty-state">Copy something to see it here!</p>
      )}
    </div>
  );
}
