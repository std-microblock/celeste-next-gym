export function atlasFrameKeys(
  keys: readonly string[],
  prefix: string,
): string[] {
  const escaped = prefix.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const matcher = new RegExp(`^${escaped}\\d+$`);
  const numbered = keys
    .filter((key) => matcher.test(key))
    .sort((left, right) =>
      left.localeCompare(right, undefined, { numeric: true }),
    );

  if (numbered.length > 0) return numbered;
  return keys.includes(prefix) ? [prefix] : [];
}
