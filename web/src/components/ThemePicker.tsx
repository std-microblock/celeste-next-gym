import { useEffect, useMemo, useRef, useState } from "react";
import type {
  VisualTheme,
  VisualThemeCollectionId,
  VisualThemeId,
} from "../visualThemes";
import themeText from "../themePickerText.json";

/**
 * Custom theme picker: expandable dropdown with free-text search, hierarchical
 * grouping (mod / category / map) and a pre-generated thumbnail preview.
 */

interface ThemePickerProps {
  themes: readonly VisualTheme[];
  collections: readonly {
    id: VisualThemeCollectionId;
    label: string;
  }[];
  value: VisualThemeId;
  onChange: (id: VisualThemeId) => void;
}

/** Chinese category label derived from the map file path (no .bin). */
export function themeGroupLabel(mapFile: string): string {
  let segments = mapFile.replace(/\.bin$/i, "").split("/");
  const collection = segments[0];
  if (collection === "StrawberryJam2021") {
    segments = segments.slice(1);
    const groups = themeText.groups as Record<string, string>;
    return groups[segments[0]] ?? segments[0] ?? themeText.otherWord;
  }
  if (collection === "ChineseNewYear2024") {
    segments = segments.slice(1);
    if (segments[0] === "0-Lobbies") return themeText.groups["0-Lobbies"];
    if (segments[0] === "1-Maps") return themeText.mapWord;
    return segments[0] ?? themeText.otherWord;
  }
  // Vanilla chapter maps are flat: "1-ForsakenCity.bin" -> ???.
  const chapter = /^(\d+)/.exec(segments[0] ?? "")?.[1];
  if (chapter !== undefined) {
    const names = themeText.chapters as Record<string, string>;
    return names[chapter] ?? themeText.chapterFallback;
  }
  return segments[0] ?? themeText.otherWord;
}

const PREVIEW_WIDTH = 144;
const PREVIEW_HEIGHT = 81;

export function ThemePicker({
  themes,
  collections,
  value,
  onChange,
}: ThemePickerProps) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [previewId, setPreviewId] = useState<VisualThemeId>(value);
  const rootRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!open) return;
    const close = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) setOpen(false);
    };
    const key = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };
    window.addEventListener("mousedown", close);
    window.addEventListener("keydown", key);
    return () => {
      window.removeEventListener("mousedown", close);
      window.removeEventListener("keydown", key);
    };
  }, [open]);

  useEffect(() => {
    if (open) searchRef.current?.focus();
  }, [open]);

  const current = themes.find((theme) => theme.id === value);

  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) return themes;
    return themes.filter(
      (theme) =>
        theme.label.toLowerCase().includes(needle) ||
        theme.chapter.toLowerCase().includes(needle) ||
        theme.collection.toLowerCase().includes(needle),
    );
  }, [themes, query]);

  return (
    <div className="theme-picker" ref={rootRef}>
      <button
        type="button"
        className="theme-picker-trigger"
        aria-label={themeText.ariaLabel}
        aria-expanded={open}
        onClick={() => setOpen((flag) => !flag)}
      >
        <span
          className="theme-picker-swatch"
          style={{ background: current?.background ?? "#000000" }}
        />
        <span className="theme-picker-trigger-label">
          {current?.label ?? themeText.themeWord}
        </span>
        <span className="theme-picker-caret">{open ? "\u25b2" : "\u25bc"}</span>
      </button>
      {open && (
        <div className="theme-picker-panel">
          <input
            ref={searchRef}
            className="theme-picker-search"
            placeholder={themeText.searchPlaceholder}
            value={query}
            onChange={(event) => setQuery(event.target.value)}
          />
          <div className="theme-picker-list">
            {collections.map((collection) => {
              const groupThemes = filtered.filter(
                (theme) => theme.collection === collection.id,
              );
              if (groupThemes.length === 0) return null;
              const byGroup = new Map<string, VisualTheme[]>();
              for (const theme of groupThemes) {
                const group = themeGroupLabel(theme.chapter);
                const list = byGroup.get(group) ?? [];
                list.push(theme);
                byGroup.set(group, list);
              }
              return (
                <div className="theme-picker-collection" key={collection.id}>
                  <div className="theme-picker-collection-label">
                    {collection.label}
                  </div>
                  {[...byGroup.entries()].map(([group, list]) => (
                    <div className="theme-picker-group" key={group}>
                      <div className="theme-picker-group-label">{group}</div>
                      {list.map((theme) => (
                        <button
                          type="button"
                          key={theme.id}
                          className={"theme-picker-option" + (theme.id === value ? " selected" : "")}
                          onMouseEnter={() => setPreviewId(theme.id)}
                          onClick={() => {
                            onChange(theme.id);
                            setOpen(false);
                          }}
                        >
                          {theme.label}
                          {theme.id === value ? " \u2713" : ""}
                        </button>
                      ))}
                    </div>
                  ))}
                </div>
              );
            })}
          </div>
          <ThemePreview theme={themes.find((theme) => theme.id === previewId) ?? null} />
        </div>
      )}
    </div>
  );
}

function ThemePreview({ theme }: { theme: VisualTheme | null }) {
  if (!theme?.previewUrl) return null;
  return (
    <div className="theme-picker-preview">
      <img
        src={"/" + theme.previewUrl}
        alt={theme.label}
        width={PREVIEW_WIDTH}
        height={PREVIEW_HEIGHT}
      />
      <small>{theme.label}</small>
    </div>
  );
}
