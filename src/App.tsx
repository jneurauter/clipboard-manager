import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

type ClipboardItem = { content: string };

export default function App() {
  const [history, setHistory] = useState<ClipboardItem[]>([]);

  const refreshHistory = async () => {
    const newHistory = await invoke<ClipboardItem[]>("get_history");
    setHistory(newHistory);
  };

  const handleCopy = async (text: string) => {
    await invoke("copy_to_clipboard", { text });
    refreshHistory();
  };

  useEffect(() => {
    refreshHistory();

    const unlisten = listen<ClipboardItem[]>("clipboard_update", (event) => {
      setHistory(event.payload);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  return (
    <div className="p-4 max-w-md mx-auto">
      <h1 className="text-xl font-bold mb-3">Clipboard Manager</h1>

      <ul>
        {history.map((item, i) => (
          <li
            key={i}
            className="border-b py-2 flex justify-between items-center"
          >
            <span>{item.content}</span>
            <button
              className="text-blue-500 hover:underline"
              onClick={() => handleCopy(item.content)}
            >
              Copy
            </button>
          </li>
        ))}
      </ul>

      {history.length === 0 && (
        <p className="text-gray-500 italic">Copy something to see it here!</p>
      )}
    </div>
  );
}
