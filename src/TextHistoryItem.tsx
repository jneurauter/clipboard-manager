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
  return (
    <li className="history-item history-item--text">
      <div className="history-item-row">
        <button
          type="button"
          className={`btn-expand${expanded ? " btn-expand--open" : ""}`}
          aria-expanded={expanded}
          aria-label={expanded ? "Collapse full text" : "Show full text"}
          onClick={onToggleExpand}
        >
          ▶
        </button>
        <span className="history-text">{content}</span>
        <button type="button" className="btn-copy" onClick={onCopy}>
          Copy
        </button>
      </div>
      {expanded && (
        <div className="history-text-full" role="region" aria-label="Full text">
          {content}
        </div>
      )}
    </li>
  );
}
