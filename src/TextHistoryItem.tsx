import { useCallback, useEffect, useRef, useState } from "react";

type Props = {
  content: string;
  expanded: boolean;
  onToggleExpand: () => void;
  onCopy: () => void;
};

export function TextHistoryItem({
  content,
  expanded,
  onToggleExpand,
  onCopy,
}: Props) {
  const textRef = useRef<HTMLSpanElement>(null);
  const [isOverflowing, setIsOverflowing] = useState(false);

  const checkOverflow = useCallback(() => {
    const el = textRef.current;
    if (!el) {
      return;
    }
    setIsOverflowing(el.scrollWidth > el.clientWidth);
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
        {showExpand && (
          <button
            type="button"
            className={`btn-expand${expanded ? " btn-expand--open" : ""}`}
            aria-expanded={expanded}
            aria-label={expanded ? "Collapse full text" : "Show full text"}
            onClick={onToggleExpand}
          >
            ▶
          </button>
        )}
        <span ref={textRef} className="history-text">
          {content}
        </span>
        <button type="button" className="btn-copy" onClick={onCopy}>
          Copy
        </button>
      </div>
      {expanded && (
        <div
          className={`history-text-full${showExpand ? "" : " history-text-full--no-indent"}`}
          role="region"
          aria-label="Full text"
        >
          {content}
        </div>
      )}
    </li>
  );
}
