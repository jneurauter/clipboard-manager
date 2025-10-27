import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type ClipboardItem = { content: string };

export default function App() {
  const [history, setHistory] = useState<ClipboardItem[]>([]);

  useEffect(() => {
    invoke<ClipboardItem[]>("get_history").then(setHistory);
  }, []);

  const handleCopy = async (text: string) => {
    await invoke("copy_to_clipboard", { text });
  };

  return (
    <div className="p-4 max-w-md mx-auto">
      <h1 className="text-xl font-bold mb-3">Clipboard Manager</h1>
      <ul>
        {history.map((item, i) => (
          <li key={i} className="border-b py-2">
            <p>{item.content}</p>
            <button
              className="text-blue-500 hover:underline"
              onClick={() => handleCopy(item.content)}
            >
              Copy
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
