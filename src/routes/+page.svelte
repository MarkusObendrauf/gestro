<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
  import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
  import GestroWheel from '$lib/GestroWheel.svelte';
  import ShortcutRecorder from '$lib/ShortcutRecorder.svelte';
  import type { Config, Direction } from '$lib/types';
  import { defaultConfig } from '$lib/types';

  let config = $state<Config>(defaultConfig());
  let selected = $state<Direction | null>(null);
  let saved = $state(false);
  let saveError = $state<string | null>(null);
  let autostart = $state(false);

  onMount(async () => {
    try {
      config = await invoke<Config>('get_config');
    } catch (e) {
      console.error('Failed to load config', e);
    }
    try {
      autostart = await isEnabled();
    } catch (e) {
      console.error('Failed to load autostart state', e);
    }

    // Fit window to content
    await tick();
    const h = appEl!.scrollHeight;
    const w = appEl!.scrollWidth;
    const win = getCurrentWindow();
    await win.setMinSize(new LogicalSize(w, h));
    const current = await win.outerSize();
    const sf = await win.scaleFactor();
    const currentH = current.height / sf;
    if (currentH < h) {
      await win.setSize(new LogicalSize(current.width / sf, h));
    }
  });

  async function toggleAutostart() {
    try {
      if (autostart) {
        await disable();
        autostart = false;
      } else {
        await enable();
        autostart = true;
      }
    } catch (e) {
      console.error('Failed to toggle autostart', e);
    }
  }

  function openRecorder(dir: Direction) {
    selected = dir;
  }

  function handleConfirm(dir: Direction, keys: string[] | null) {
    config = {
      ...config,
      directions: {
        ...config.directions,
        [dir]: keys ? { keys } : null,
      },
    };
    selected = null;
  }

  async function handleSave() {
    saveError = null;
    try {
      await invoke('save_config', { newConfig: config });
      saved = true;
      setTimeout(() => (saved = false), 2000);
    } catch (e) {
      saveError = String(e);
    }
  }

  let boundCount = $derived(
    Object.values(config.directions).filter((v) => v !== null).length
  );

  let appEl = $state<HTMLDivElement | null>(null);
</script>

<div class="app" bind:this={appEl}>
  <header>
    <div class="brand">gestro</div>
    <div class="meta">{boundCount} / 8 gestures bound</div>
  </header>

  <main>
    <GestroWheel {config} onSelect={openRecorder} />

    <div class="controls">
      <label class="threshold-row">
        <span>Threshold</span>
        <input
          type="range"
          min="5"
          max="60"
          step="1"
          bind:value={config.threshold_px}
        />
        <span class="threshold-val">{config.threshold_px}px</span>
      </label>

      <label class="autostart-row">
        <input type="checkbox" checked={autostart} onchange={toggleAutostart} />
        <span>Launch at login</span>
      </label>

      <button class="save-btn" class:saved onclick={handleSave}>
        {saved ? '✓ Saved' : 'Save'}
      </button>
    </div>

    {#if saveError}
      <div class="error">{saveError}</div>
    {/if}
  </main>

  {#if selected !== null}
    <ShortcutRecorder
      direction={selected}
      current={config.directions[selected]?.keys ?? null}
      onConfirm={(keys) => handleConfirm(selected!, keys)}
      onCancel={() => (selected = null)}
    />
  {/if}
</div>

<style>
  :global(*, *::before, *::after) { box-sizing: border-box; }
  :global(body) {
    margin: 0;
    background: #111111;
    color: #e0e0e0;
    font-family: system-ui, -apple-system, sans-serif;
    font-size: 14px;
    overflow: hidden;
  }

  .app {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 24px;
    border-bottom: 1px solid #1e1e1e;
    flex-shrink: 0;
  }

  .brand {
    font-size: 20px;
    font-weight: 800;
    letter-spacing: 2px;
    color: #4fc3f7;
  }

  .meta {
    font-size: 12px;
    color: #555555;
    letter-spacing: 0.3px;
  }

  main {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 20px;
    padding: 20px;
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 20px;
    background: #161616;
    border: 1px solid #222222;
    border-radius: 10px;
    padding: 12px 20px;
    width: 400px;
  }

  .threshold-row {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 1;
    font-size: 13px;
    color: #888888;
  }

  input[type='range'] {
    flex: 1;
    accent-color: #4fc3f7;
    height: 4px;
    cursor: pointer;
  }

  .threshold-val {
    color: #e0e0e0;
    font-size: 12px;
    min-width: 32px;
    text-align: right;
  }

  .autostart-row {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 13px;
    color: #888888;
    cursor: pointer;
    flex-shrink: 0;
    white-space: nowrap;
  }

  .autostart-row input[type='checkbox'] {
    accent-color: #4fc3f7;
    width: 14px;
    height: 14px;
    cursor: pointer;
  }

  .save-btn {
    padding: 7px 20px;
    background: #1a4a5e;
    color: #4fc3f7;
    border: 1px solid #2a6a7e;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
    flex-shrink: 0;
  }

  .save-btn:hover { background: #1e5570; }

  .save-btn.saved {
    background: #1a3a20;
    color: #6fcf97;
    border-color: #2a5a30;
  }

  .error {
    color: #f07070;
    font-size: 12px;
    max-width: 400px;
    text-align: center;
  }
</style>
