import { Mouse, Play, Waves } from "lucide-react";
import { api } from "../lib/tauri";
import { useAppStore } from "../store/useAppStore";
import { ToggleSwitch } from "./ToggleSwitch";

export function MouseScrollTab() {
  const { config, sprites, patchConfig } = useAppStore();
  if (!config) return null;

  const mouseSprites = sprites.filter((s) =>
    ["Mouse", "Gamepad", "Hardware", "Tactile"].includes(s.category),
  );
  const scrollSprites = sprites.filter((s) =>
    ["Scroll", "Hardware", "Tactile"].includes(s.category),
  );

  return (
    <div className="space-y-5">
      <div>
        <h2 className="text-lg font-semibold tracking-tight">Mouse &amp; Scroll Sounds</h2>
        <p className="text-xs text-zinc-400">
          Add crisp mechanical mouse click acoustics (Klack 1.7) and tactile rotary scroll ticks (Klack 2.2).
        </p>
      </div>

      {/* Mouse Clicks */}
      <div className="space-y-4 rounded-2xl border border-zinc-800/80 bg-zinc-900/50 p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2.5">
            <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-amber-500/15 text-amber-400">
              <Mouse className="h-4 w-4" />
            </div>
            <div>
              <div className="text-sm font-semibold text-zinc-100">Mouse Click Sounds</div>
              <div className="text-[11px] text-zinc-400">
                Distinct press &amp; release samples for Left, Right, and Middle clicks
              </div>
            </div>
          </div>
          <ToggleSwitch
            checked={config.mouse.enabled}
            onChange={(enabled) =>
              patchConfig((c) => ({ ...c, mouse: { ...c.mouse, enabled } }))
            }
          />
        </div>

        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          <div>
            <label className="mb-1 block text-[11px] text-zinc-400">
              Primary Click Sound (Left / Right)
            </label>
            <div className="flex gap-1.5">
              <select
                value={config.mouse.sound_id}
                onChange={(e) => {
                  const sound_id = e.target.value;
                  patchConfig((c) => ({ ...c, mouse: { ...c.mouse, sound_id } }));
                }}
                className="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-2.5 py-1.5 text-xs text-zinc-100 focus:border-amber-500 focus:outline-none"
              >
                {mouseSprites.map((s) => (
                  <option key={s.id} value={s.id}>
                    {s.label}
                  </option>
                ))}
              </select>
              <button
                type="button"
                onClick={() => api.previewSprite(config.mouse.sound_id, config.mouse.volume)}
                className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-zinc-700 bg-zinc-800 text-zinc-200 hover:text-amber-300"
                title="Test click sound"
              >
                <Play className="h-3.5 w-3.5 fill-current" />
              </button>
            </div>
          </div>

          <div>
            <label className="mb-1 block text-[11px] text-zinc-400">
              Middle Click Sound
            </label>
            <div className="flex gap-1.5">
              <select
                value={config.mouse.middle_sound_id}
                onChange={(e) => {
                  const middle_sound_id = e.target.value;
                  patchConfig((c) => ({
                    ...c,
                    mouse: { ...c.mouse, middle_sound_id },
                  }));
                }}
                className="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-2.5 py-1.5 text-xs text-zinc-100 focus:border-amber-500 focus:outline-none"
              >
                {mouseSprites.map((s) => (
                  <option key={s.id} value={s.id}>
                    {s.label}
                  </option>
                ))}
              </select>
              <button
                type="button"
                onClick={() =>
                  api.previewSprite(config.mouse.middle_sound_id, config.mouse.volume)
                }
                className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-zinc-700 bg-zinc-800 text-zinc-200 hover:text-amber-300"
                title="Test middle click"
              >
                <Play className="h-3.5 w-3.5 fill-current" />
              </button>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 items-center">
          <div>
            <div className="mb-1 flex justify-between text-[11px] text-zinc-400">
              <span>Mouse Volume</span>
              <span className="font-mono">{Math.round(config.mouse.volume * 100)}%</span>
            </div>
            <input
              type="range"
              min={0.1}
              max={1.2}
              step={0.05}
              value={config.mouse.volume}
              onChange={(e) => {
                const volume = parseFloat(e.target.value);
                patchConfig((c) => ({ ...c, mouse: { ...c.mouse, volume } }));
              }}
              className="w-full"
            />
          </div>

          <div className="flex items-center justify-between rounded-xl border border-zinc-800 bg-zinc-950/60 px-3 py-2">
            <span className="text-xs text-zinc-300">Button Release Sound</span>
            <ToggleSwitch
              checked={config.mouse.play_release}
              onChange={(play_release) =>
                patchConfig((c) => ({ ...c, mouse: { ...c.mouse, play_release } }))
              }
            />
          </div>
        </div>
      </div>

      {/* Scroll Wheel Sounds */}
      <div className="space-y-4 rounded-2xl border border-zinc-800/80 bg-zinc-900/50 p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2.5">
            <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-sky-500/15 text-sky-400">
              <Waves className="h-4 w-4" />
            </div>
            <div>
              <div className="text-sm font-semibold text-zinc-100">Scroll Wheel Ticks</div>
              <div className="text-[11px] text-zinc-400">
                Rotary encoder clicks with velocity-scaled pitch and rate limiting
              </div>
            </div>
          </div>
          <ToggleSwitch
            checked={config.scroll.enabled}
            onChange={(enabled) =>
              patchConfig((c) => ({ ...c, scroll: { ...c.scroll, enabled } }))
            }
          />
        </div>

        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          <div>
            <label className="mb-1 block text-[11px] text-zinc-400">
              Scroll Tick Sound
            </label>
            <div className="flex gap-1.5">
              <select
                value={config.scroll.sound_id}
                onChange={(e) => {
                  const sound_id = e.target.value;
                  patchConfig((c) => ({ ...c, scroll: { ...c.scroll, sound_id } }));
                }}
                className="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-2.5 py-1.5 text-xs text-zinc-100 focus:border-amber-500 focus:outline-none"
              >
                {scrollSprites.map((s) => (
                  <option key={s.id} value={s.id}>
                    {s.label}
                  </option>
                ))}
              </select>
              <button
                type="button"
                onClick={() => api.previewSprite(config.scroll.sound_id, config.scroll.volume)}
                className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-zinc-700 bg-zinc-800 text-zinc-200 hover:text-amber-300"
                title="Test scroll tick"
              >
                <Play className="h-3.5 w-3.5 fill-current" />
              </button>
            </div>
          </div>

          <div>
            <div className="mb-1 flex justify-between text-[11px] text-zinc-400">
              <span>Scroll Volume</span>
              <span className="font-mono">{Math.round(config.scroll.volume * 100)}%</span>
            </div>
            <input
              type="range"
              min={0.1}
              max={1.2}
              step={0.05}
              value={config.scroll.volume}
              onChange={(e) => {
                const volume = parseFloat(e.target.value);
                patchConfig((c) => ({ ...c, scroll: { ...c.scroll, volume } }));
              }}
              className="w-full"
            />
          </div>
        </div>

        <div>
          <div className="mb-1 flex justify-between text-[11px] text-zinc-400">
            <span>Tick Spacing (Debounce)</span>
            <span className="font-mono">{config.scroll.min_interval_ms} ms</span>
          </div>
          <input
            type="range"
            min={20}
            max={120}
            step={2}
            value={config.scroll.min_interval_ms}
            onChange={(e) => {
              const min_interval_ms = parseInt(e.target.value, 10);
              patchConfig((c) => ({
                ...c,
                scroll: { ...c.scroll, min_interval_ms },
              }));
            }}
            className="w-full"
          />
        </div>
      </div>
    </div>
  );
}
