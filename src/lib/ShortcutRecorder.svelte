<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { Direction } from './types';
  import { formatShortcut } from './types';

  let {
    direction,
    current,
    onConfirm,
    onCancel,
  }: {
    direction: Direction;
    current: string[] | null;
    onConfirm: (keys: string[] | null) => void;
    onCancel: () => void;
  } = $props();

  let recording = $state(false);
  let captured = $state<string[]>(current ?? []);

  const MODIFIER_KEYS = new Set(['Control', 'Shift', 'Alt', 'Meta']);

  function normalizeKey(key: string): string {
    const map: Record<string, string> = {
      ' ': 'space', 'Escape': 'esc', 'Enter': 'enter', 'Backspace': 'backspace',
      'Delete': 'delete', 'Tab': 'tab', 'ArrowUp': 'up', 'ArrowDown': 'down',
      'ArrowLeft': 'left', 'ArrowRight': 'right', 'Home': 'home', 'End': 'end',
      'PageUp': 'pageup', 'PageDown': 'pagedown', 'Insert': 'insert',
      'PrintScreen': 'print', 'CapsLock': 'capslock', 'NumLock': 'numlock',
    };
    if (map[key]) return map[key];
    if (/^F\d{1,2}$/.test(key)) return key.toLowerCase();
    return key.toLowerCase();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!recording) return;
    e.preventDefault();
    e.stopPropagation();

    if (MODIFIER_KEYS.has(e.key)) return;

    const keys: string[] = [];
    if (e.ctrlKey) keys.push('ctrl');
    if (e.shiftKey) keys.push('shift');
    if (e.altKey) keys.push('alt');
    if (e.metaKey) keys.push('super');
    keys.push(normalizeKey(e.key));

    captured = keys;
    recording = false;
  }

  onMount(() => window.addEventListener('keydown', handleKeydown, true));
  onDestroy(() => window.removeEventListener('keydown', handleKeydown, true));
</script>

<!-- Backdrop -->
<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onCancel}>
  <!-- Card — stop propagation so clicks inside don't dismiss -->
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="card" onclick={(e) => e.stopPropagation()}>
    <div class="dir-badge">{direction}</div>
    <h2>Assign shortcut</h2>

    <div class="display" class:recording>
      {#if recording}
        <span class="hint">Press a key combination…</span>
      {:else if captured.length > 0}
        <span class="keys">{formatShortcut(captured)}</span>
      {:else}
        <span class="empty">No shortcut</span>
      {/if}
    </div>

    <div class="actions">
      {#if recording}
        <button class="btn secondary" onclick={() => (recording = false)}>Cancel recording</button>
      {:else}
        <button class="btn primary" onclick={() => { recording = true; captured = []; }}>
          {captured.length > 0 ? 'Re-record' : 'Record'}
        </button>
        {#if captured.length > 0}
          <button class="btn danger" onclick={() => (captured = [])}>Clear</button>
        {/if}
      {/if}
    </div>

    <div class="footer">
      <button class="btn ghost" onclick={onCancel}>Cancel</button>
      <button
        class="btn confirm"
        onclick={() => onConfirm(captured.length > 0 ? captured : null)}
      >Save</button>
    </div>
  </div>
</div>

<style>
.backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.65);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.card {
  background: #1c1c1c;
  border: 1px solid #2e2e2e;
  border-radius: 12px;
  padding: 28px 32px;
  width: 320px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.6);
}

.dir-badge {
  align-self: flex-start;
  background: #132a38;
  border: 1px solid #2a5a6a;
  color: #4fc3f7;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 1.5px;
  padding: 3px 10px;
  border-radius: 4px;
}

h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #e0e0e0;
}

.display {
  background: #111111;
  border: 1px solid #2a2a2a;
  border-radius: 8px;
  padding: 16px;
  min-height: 52px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: border-color 0.15s;
}

.display.recording {
  border-color: #4fc3f7;
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%, 100% { border-color: #4fc3f7; }
  50% { border-color: #1a5a7a; }
}

.keys {
  font-family: 'SF Mono', 'Fira Mono', monospace;
  font-size: 18px;
  font-weight: 600;
  color: #4fc3f7;
  letter-spacing: 0.5px;
}

.hint {
  font-size: 13px;
  color: #666666;
  font-style: italic;
}

.empty {
  font-size: 13px;
  color: #444444;
}

.actions {
  display: flex;
  gap: 8px;
}

.footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding-top: 4px;
  border-top: 1px solid #222222;
}

.btn {
  padding: 7px 16px;
  border-radius: 6px;
  border: none;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.12s, opacity 0.12s;
}

.btn.primary {
  background: #1a4a5e;
  color: #4fc3f7;
  border: 1px solid #2a6a7e;
}
.btn.primary:hover { background: #1e5570; }

.btn.secondary {
  background: #252525;
  color: #aaaaaa;
  border: 1px solid #333333;
}
.btn.secondary:hover { background: #2e2e2e; }

.btn.danger {
  background: #3a1a1a;
  color: #f07070;
  border: 1px solid #5a2a2a;
}
.btn.danger:hover { background: #4a2020; }

.btn.ghost {
  background: transparent;
  color: #666666;
  border: 1px solid #2a2a2a;
}
.btn.ghost:hover { color: #aaaaaa; border-color: #3a3a3a; }

.btn.confirm {
  background: #4fc3f7;
  color: #0a1a22;
  font-weight: 700;
}
.btn.confirm:hover { background: #6dd0f9; }
</style>
