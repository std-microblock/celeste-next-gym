import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import themeText from "../themePickerText.json";
import { VISUAL_THEME_COLLECTIONS, VISUAL_THEMES } from "../visualThemes";
import { ThemePicker, themeGroupLabel } from "./ThemePicker";

describe("ThemePicker", () => {
  it("derives Chinese category labels from map paths", () => {
    const groups = themeText.groups as Record<string, string>;
    const chapters = themeText.chapters as Record<string, string>;
    expect(themeGroupLabel("1-ForsakenCity.bin")).toBe(chapters["1"]);
    expect(themeGroupLabel("StrawberryJam2021/0-Gyms/1-Beginner.bin")).toBe(groups["0-Gyms"]);
    expect(themeGroupLabel("StrawberryJam2021/1-Beginner/mosscairn.bin")).toBe(groups["1-Beginner"]);
    expect(themeGroupLabel("StrawberryJam2021/0-Lobbies/1-Beginner.bin")).toBe(groups["0-Lobbies"]);
    expect(themeGroupLabel("ChineseNewYear2024/1-Maps/fengchen.bin")).toBe(themeText.mapWord);
  });

  it("opens, searches and selects a theme", () => {
    const onChange = vi.fn();
    const forsaken = VISUAL_THEMES.find(
      (theme) => theme.chapter === "1-ForsakenCity.bin",
    )!;
    render(
      <ThemePicker
        themes={VISUAL_THEMES}
        collections={VISUAL_THEME_COLLECTIONS}
        value={VISUAL_THEMES[0].id}
        onChange={onChange}
      />,
    );
    fireEvent.click(screen.getByLabelText(themeText.ariaLabel));
    fireEvent.change(screen.getByPlaceholderText(themeText.searchPlaceholder), {
      target: { value: forsaken.label },
    });
    const option = screen.getByRole("button", { name: forsaken.label });
    fireEvent.click(option);
    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange.mock.calls[0][0]).toContain("1-ForsakenCity");
  });
});
