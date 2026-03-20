<script lang="ts">
  import type { Direction, Config } from './types';
  import { formatShortcut } from './types';

  let {
    config,
    onSelect,
  }: {
    config: Config;
    onSelect: (dir: Direction) => void;
  } = $props();

  const CX = 200;
  const CY = 200;
  const R_INNER = 58;
  const R_OUTER = 178;
  const R_DIR_LABEL = 152;
  const R_KEY_LABEL = 118;
  const GAP_DEG = 1.8;

  // SVG angle (0=right, clockwise): direction → center angle
  const ANGLES: Record<Direction, number> = {
    N: 270, NE: 315, E: 0, SE: 45,
    S: 90, SW: 135, W: 180, NW: 225,
  };

  const DIRS: Direction[] = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW'];

  let hovered = $state<Direction | null>(null);

  function toRad(deg: number) { return (deg * Math.PI) / 180; }

  function sectorPath(centerDeg: number): string {
    const a1 = toRad(centerDeg - 22.5 + GAP_DEG);
    const a2 = toRad(centerDeg + 22.5 - GAP_DEG);
    const ox1 = CX + R_OUTER * Math.cos(a1);
    const oy1 = CY + R_OUTER * Math.sin(a1);
    const ox2 = CX + R_OUTER * Math.cos(a2);
    const oy2 = CY + R_OUTER * Math.sin(a2);
    const ix1 = CX + R_INNER * Math.cos(a2);
    const iy1 = CY + R_INNER * Math.sin(a2);
    const ix2 = CX + R_INNER * Math.cos(a1);
    const iy2 = CY + R_INNER * Math.sin(a1);
    return `M ${ox1} ${oy1} A ${R_OUTER} ${R_OUTER} 0 0 1 ${ox2} ${oy2} L ${ix1} ${iy1} A ${R_INNER} ${R_INNER} 0 0 0 ${ix2} ${iy2} Z`;
  }

  function labelPos(r: number, centerDeg: number) {
    const a = toRad(centerDeg);
    return { x: CX + r * Math.cos(a), y: CY + r * Math.sin(a) };
  }

  function sectorFill(dir: Direction): string {
    const bound = config.directions[dir] !== null;
    if (hovered === dir) return bound ? '#1a3a4a' : '#252525';
    return bound ? '#132a38' : '#1a1a1a';
  }

  function sectorStroke(dir: Direction): string {
    if (hovered === dir) return '#4fc3f7';
    return config.directions[dir] !== null ? '#2a5a6a' : '#2a2a2a';
  }
</script>

<svg
  viewBox="0 0 400 400"
  width="400"
  height="400"
  role="img"
  aria-label="Pie gesture wheel"
>
  <!-- Sectors -->
  {#each DIRS as dir}
    {@const angle = ANGLES[dir]}
    {@const shortcut = config.directions[dir]}
    {@const dirPos = labelPos(R_DIR_LABEL, angle)}
    {@const keyPos = labelPos(R_KEY_LABEL, angle)}

    <path
      d={sectorPath(angle)}
      fill={sectorFill(dir)}
      stroke={sectorStroke(dir)}
      stroke-width="1"
      style="cursor:pointer; transition: fill 0.12s, stroke 0.12s;"
      role="button"
      tabindex="0"
      aria-label="{dir}: {shortcut ? formatShortcut(shortcut.keys) : 'unbound'}"
      onmouseenter={() => (hovered = dir)}
      onmouseleave={() => (hovered = null)}
      onclick={() => onSelect(dir)}
      onkeydown={(e) => e.key === 'Enter' && onSelect(dir)}
    />

    <!-- Direction label -->
    <text
      x={dirPos.x}
      y={dirPos.y}
      text-anchor="middle"
      dominant-baseline="middle"
      font-family="system-ui, sans-serif"
      font-size="13"
      font-weight="600"
      fill={hovered === dir ? '#ffffff' : '#cccccc'}
      style="pointer-events:none; transition: fill 0.12s; letter-spacing: 0.5px;"
    >{dir}</text>

    <!-- Shortcut label -->
    {#if shortcut}
      <text
        x={keyPos.x}
        y={keyPos.y}
        text-anchor="middle"
        dominant-baseline="middle"
        font-family="system-ui, sans-serif"
        font-size="10"
        fill={hovered === dir ? '#7dd8f5' : '#4fc3f7'}
        style="pointer-events:none; transition: fill 0.12s;"
      >{formatShortcut(shortcut.keys)}</text>
    {:else}
      <text
        x={keyPos.x}
        y={keyPos.y}
        text-anchor="middle"
        dominant-baseline="middle"
        font-family="system-ui, sans-serif"
        font-size="11"
        fill={hovered === dir ? '#555555' : '#383838'}
        style="pointer-events:none;"
      >—</text>
    {/if}
  {/each}

  <!-- Center circle -->
  <circle cx={CX} cy={CY} r={R_INNER - 6} fill="#111111" stroke="#222222" stroke-width="1" />
  <text
    x={CX} y={CY - 6}
    text-anchor="middle"
    dominant-baseline="middle"
    font-family="system-ui, sans-serif"
    font-size="16"
    font-weight="700"
    fill="#333333"
  >pie</text>
  <text
    x={CX} y={CY + 10}
    text-anchor="middle"
    dominant-baseline="middle"
    font-family="system-ui, sans-serif"
    font-size="8"
    fill="#2a2a2a"
  >click to assign</text>
</svg>
