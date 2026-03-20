export type Direction = 'N' | 'NE' | 'E' | 'SE' | 'S' | 'SW' | 'W' | 'NW';

export interface Shortcut {
  keys: string[];
}

export interface Config {
  threshold_px: number;
  directions: Record<Direction, Shortcut | null>;
}

export const DIRECTIONS: Direction[] = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW'];

export function defaultConfig(): Config {
  const directions = Object.fromEntries(
    DIRECTIONS.map((d) => [d, null])
  ) as Record<Direction, Shortcut | null>;
  return { threshold_px: 15, directions };
}

export function formatShortcut(keys: string[]): string {
  return keys
    .map((k) => k.charAt(0).toUpperCase() + k.slice(1))
    .join('+');
}
