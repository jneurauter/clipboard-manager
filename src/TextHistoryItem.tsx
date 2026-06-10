import { useCallback, useEffect, useRef, useState } from "react";
import { Copy } from "lucide-react";

type Props = {
  content: string;
  onCopy: () => void;
};

export function TextHistoryItem({ content, onCopy }: Props) {
  const textRef = useRef<HTMLSpanElement>(null);
  const [isOverflowing, setIsOverflowing] = useState(false);
  const [expanded, setExpanded] = useState(false);

  const checkOverflow = useCallback(() => {
    const el = textRef.current;
    if (!el) {
      return;
    }
    setIsOverflowing(el.scrollHeight > el.clientHeight || el.scrollWidth > el.clientWidth);
  }, []);

  useEffect(() => {
    checkOverflow();
    window.addEventListener("resize", checkOverflow);
    return () => window.removeEventListener("resize", checkOverflow);
  }, [content, checkOverflow]);

  useEffect(() => {
    const el = textRef.current;
    if (!el) {
      return;
    }
    const observer = new ResizeObserver(checkOverflow);
    observer.observe(el);
    return () => observer.disconnect();
  }, [checkOverflow]);

  const showExpand = isOverflowing || expanded;

  return (
    <li className="history-item history-item--text">
      <div className="history-item-row">
        <button
          type="button"
          className="btn-copy btn-copy-left"
          onClick={onCopy}
          aria-label="Copy item"
        >
          <Copy className="copy-icon" aria-hidden="true" />
        </button>

        {showExpand && (
          <button
            type="button"
            className={`btn-expand${expanded ? " btn-expand--open" : ""}`}
            aria-expanded={expanded}
            aria-label={expanded ? "Collapse text" : "Expand text"}
            onClick={() => setExpanded((prev) => !prev)}
          >
            ▶
          </button>
        )}

        <span
          ref={textRef}
          className={`history-text${expanded ? " history-text--expanded" : ""}`}
        >
          {content}
        </span>
      </div>
    </li>
  );
}
