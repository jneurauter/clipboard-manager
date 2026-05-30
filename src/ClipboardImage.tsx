import { useCallback, useEffect, useRef, useState } from "react";

type Props = {
  dataUrl: string;
  listLength: number;
};

export function ClipboardImage({ dataUrl, listLength }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const naturalSizeRef = useRef<{ width: number; height: number } | null>(null);
  const [size, setSize] = useState<{ width: number; height: number } | null>(
    null,
  );

  const updateSize = useCallback(() => {
    const natural = naturalSizeRef.current;
    const container = containerRef.current;
    if (!natural || !container || container.clientWidth === 0) {
      return;
    }

    const maxWidth = container.clientWidth;
    const top = container.getBoundingClientRect().top;
    const maxHeight = Math.max(48, window.innerHeight - top - 16);
    const scale = Math.min(
      maxWidth / natural.width,
      maxHeight / natural.height,
    );

    setSize({
      width: Math.round(natural.width * scale),
      height: Math.round(natural.height * scale),
    });
  }, []);

  useEffect(() => {
    window.addEventListener("resize", updateSize);
    return () => window.removeEventListener("resize", updateSize);
  }, [updateSize]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) {
      return;
    }
    const observer = new ResizeObserver(updateSize);
    observer.observe(container);
    return () => observer.disconnect();
  }, [updateSize]);

  useEffect(() => {
    updateSize();
  }, [listLength, updateSize]);

  return (
    <div ref={containerRef} className="clipboard-image-container">
      <img
        src={dataUrl}
        alt="Clipboard image"
        className="clipboard-image"
        style={
          size
            ? { width: size.width, height: size.height }
            : { maxWidth: "100%", visibility: "hidden" }
        }
        onLoad={(event) => {
          const img = event.currentTarget;
          naturalSizeRef.current = {
            width: img.naturalWidth,
            height: img.naturalHeight,
          };
          img.style.visibility = "visible";
          updateSize();
        }}
      />
    </div>
  );
}
