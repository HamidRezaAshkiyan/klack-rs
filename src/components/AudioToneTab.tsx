import { useRef } from "react";
import { RefreshCw, RotateCcw, Speaker, Sparkles, Volume2 } from "lucide-react";
import { api } from "../lib/tauri";
import { useAppStore } from "../store/useAppStore";
import { ToggleSwitch } from "./ToggleSwitch";

export function AudioToneTab() {
  const { config, audioDevices, patchConfig, refreshAudioDevices } = useAppStore();
  const padRef = useRef<HTMLDivElement>(null);

  if (!config) return null;

  const updateToneFromPointer = (clientX: number, clientY: number, preview: boolean) => {
    const el = padRef.current;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    const nx = ((clientX - rect.left) / rect.width) * 2 - 1;
    const ny = -(((clientY - rect.top) / rect.height) * 2 - 1);
    const x = Math.max(-1, Math.min(1, Number(nx.toFixed(2))));
    const y = Math.max(-1, Math.min(1, Number(ny.toFixed(2))));
    patchConfig((c) => ({ ...c, tone_pad: { x, y } }));
    if (preview) {
      api.previewSwitch(config.active_switch_id);
    }
  };

  const handlePadPointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    e.currentTarget.setPointerCapture(e.pointerId);
    updateToneFromPointer(e.clientX, e.clientY, false);
  };

  const handlePadPointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    if (e.buttons !== 1) return;
    updateToneFromPointer(e.clientX, e.clientY, false);
  };

  const handlePadPointerUp = (e: React.PointerEvent<HTMLDivElement>) => {
    updateToneFromPointer(e.clientX, e.clientY, true);
  };

  const dotLeft = `${((config.tone_pad.x + 1) / 2) * 100}%`;
  const dotTop = `${((1 - config.tone_pad.y) / 2) * 100}%`;

  return (
    <div className="space-y-5">
      <div>
        <h2 className="text-lg font-semibold tracking-tight">Audio &amp; Tone Pad</h2>
        <p className="text-xs text-zinc-400">
          Shape switch acoustics in real time, route to any audio output, and tune spatial stereo depth.
        </p>
      </div>

      {/* Master Volume + Output Device */}
      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <div className="rounded-2xl border border-zinc-800/80 bg-zinc-900/50 p-4">
          <div className="mb-2 flex items-center justify-between text-xs font-medium">
            <span className="flex items-center gap-1.5 text-zinc-200">
              <Volume2 className="h-4 w-4 text-amber-400" />
              Master Volume
            </span>
            <span className="font-mono text-zinc-400">
              {Math.round(config.volume * 100)}%
            </span>
          </div>
          <input
            type="range"
            min={0}
            max={1.25}
            step={0.01}
            value={config.volume}
            onChange={(e) => {
              const v = parseFloat(e.target.value);
              patchConfig((c) => ({ ...c, volume: v }));
            }}
            className="w-full"
          />
        </div>

        <div className="rounded-2xl border border-zinc-800/80 bg-zinc-900/50 p-4">
          <div className="mb-2 flex items-center justify-between text-xs font-medium">
            <span className="flex items-center gap-1.5 text-zinc-200">
              <Speaker className="h-4 w-4 text-amber-400" />
              Output Device
            </span>
            <button
              type="button"
              onClick={() => refreshAudioDevices()}
              className="text-zinc-400 hover:text-zinc-100"
              title="Refresh devices"
            >
              <RefreshCw className="h-3.5 w-3.5" />
            </button>
          </div>
          <select
            value={config.output_device ?? ""}
            onChange={(e) => {
              const val = e.target.value || null;
              patchConfig((c) => ({ ...c, output_device: val }));
            }}
            className="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-2.5 py-1.5 text-xs text-zinc-100 focus:border-amber-500 focus:outline-none"
          >
            <option value="">System Default</option>
            {audioDevices.map((d) => (
              <option key={d.name} value={d.name}>
                {d.name} {d.is_default ? "(Default)" : ""}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Interactive 2D Tone Pad (Klack 2.0) */}
      <div className="rounded-2xl border border-zinc-800/80 bg-zinc-900/50 p-4">
        <div className="mb-3 flex items-center justify-between">
          <div>
            <div className="flex items-center gap-1.5 text-sm font-medium text-zinc-100">
              <Sparkles className="h-4 w-4 text-amber-400" />
              Tone Pad
            </div>
            <p className="text-[11px] text-zinc-400">
              Drag to sculpt pitch, brightness, and punch. Release to preview.
            </p>
          </div>
          <button
            type="button"
            onClick={() =>
              patchConfig((c) => ({ ...c, tone_pad: { x: 0, y: 0 } }))
            }
            className="inline-flex items-center gap-1 rounded-lg border border-zinc-700 px-2.5 py-1 text-[11px] text-zinc-300 hover:bg-zinc-800"
          >
            <RotateCcw className="h-3 w-3" />
            Reset
          </button>
        </div>

        <div
          ref={padRef}
          onPointerDown={handlePadPointerDown}
          onPointerMove={handlePadPointerMove}
          onPointerUp={handlePadPointerUp}
          className="relative h-44 w-full cursor-crosshair overflow-hidden rounded-xl border border-zinc-700/80 bg-gradient-to-tr from-zinc-950 via-zinc-900 to-amber-500/10"
        >
          {/* Crosshair grid */}
          <div className="pointer-events-none absolute inset-x-0 top-1/2 h-px bg-zinc-700/50" />
          <div className="pointer-events-none absolute inset-y-0 left-1/2 w-px bg-zinc-700/50" />

          {/* Axis labels */}
          <span className="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-[10px] font-medium uppercase tracking-wider text-zinc-500">
            Warm / Thock
          </span>
          <span className="pointer-events-none absolute right-2.5 top-1/2 -translate-y-1/2 text-[10px] font-medium uppercase tracking-wider text-zinc-500">
            Crisp / Clack
          </span>
          <span className="pointer-events-none absolute left-1/2 top-2 -translate-x-1/2 text-[10px] font-medium uppercase tracking-wider text-zinc-500">
            Punchy
          </span>
          <span className="pointer-events-none absolute bottom-2 left-1/2 -translate-x-1/2 text-[10px] font-medium uppercase tracking-wider text-zinc-500">
            Damped
          </span>

          {/* Draggable Puck */}
          <div
            style={{ left: dotLeft, top: dotTop }}
            className="pointer-events-none absolute h-5 w-5 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-white bg-amber-500 shadow-[0_0_20px_rgba(245,158,11,0.7)]"
          />
        </div>

        <div className="mt-2 flex justify-between font-mono text-[11px] text-zinc-500">
          <span>Tone X: {config.tone_pad.x.toFixed(2)}</span>
          <span>Punch Y: {config.tone_pad.y.toFixed(2)}</span>
        </div>
      </div>

      {/* Spatial Audio & Pitch Randomization */}
      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <div className="space-y-3 rounded-2xl border border-zinc-800/80 bg-zinc-900/50 p-4">
          <div className="flex items-center justify-between">
            <div>
              <div className="text-xs font-semibold text-zinc-100">Spatial Audio</div>
              <div className="text-[11px] text-zinc-400">
                3D left-to-right stereo placement per key
              </div>
            </div>
            <ToggleSwitch
              checked={config.spatial.enabled}
              onChange={(enabled) =>
                patchConfig((c) => ({ ...c, spatial: { ...c.spatial, enabled } }))
              }
            />
          </div>

          <div>
            <div className="mb-1 flex justify-between text-[11px] text-zinc-400">
              <span>Stereo Width</span>
              <span className="font-mono">{Math.round(config.spatial.width * 100)}%</span>
            </div>
            <input
              type="range"
              min={0}
              max={1}
              step={0.05}
              disabled={!config.spatial.enabled}
              value={config.spatial.width}
              onChange={(e) => {
                const width = parseFloat(e.target.value);
                patchConfig((c) => ({ ...c, spatial: { ...c.spatial, width } }));
              }}
              className="w-full"
            />
          </div>

          <div className="flex items-center justify-between pt-1 text-xs">
            <span className="text-zinc-300">Headphone ITD Micro-Delay</span>
            <ToggleSwitch
              checked={config.spatial.itd_enabled}
              disabled={!config.spatial.enabled}
              onChange={(itd_enabled) =>
                patchConfig((c) => ({ ...c, spatial: { ...c.spatial, itd_enabled } }))
              }
            />
          </div>
        </div>

        <div className="space-y-3 rounded-2xl border border-zinc-800/80 bg-zinc-900/50 p-4">
          <div>
            <div className="mb-1 flex justify-between text-xs font-medium">
              <span className="text-zinc-200">Randomized Pitching</span>
              <span className="font-mono text-zinc-400">
                ±{(config.pitch_randomization * 100).toFixed(1)}%
              </span>
            </div>
            <input
              type="range"
              min={0}
              max={0.08}
              step={0.005}
              value={config.pitch_randomization}
              onChange={(e) => {
                const pitch_randomization = parseFloat(e.target.value);
                patchConfig((c) => ({ ...c, pitch_randomization }));
              }}
              className="w-full"
            />
          </div>

          <div className="flex items-center justify-between text-xs">
            <span className="text-zinc-200">Key-Up Release Sounds</span>
            <ToggleSwitch
              checked={config.key_categories.play_key_up}
              onChange={(play_key_up) =>
                patchConfig((c) => ({
                  ...c,
                  key_categories: { ...c.key_categories, play_key_up },
                }))
              }
            />
          </div>

          <div>
            <div className="mb-1 flex justify-between text-[11px] text-zinc-400">
              <span>Key-Up Volume Balance</span>
              <span className="font-mono">
                {Math.round(config.key_categories.key_up_volume * 100)}%
              </span>
            </div>
            <input
              type="range"
              min={0.1}
              max={1}
              step={0.02}
              disabled={!config.key_categories.play_key_up}
              value={config.key_categories.key_up_volume}
              onChange={(e) => {
                const key_up_volume = parseFloat(e.target.value);
                patchConfig((c) => ({
                  ...c,
                  key_categories: { ...c.key_categories, key_up_volume },
                }));
              }}
              className="w-full"
            />
          </div>
        </div>
      </div>

      {/* Optional Key Categories */}
      <div className="rounded-2xl border border-zinc-800/80 bg-zinc-900/50 p-4">
        <div className="mb-3 text-xs font-semibold text-zinc-200">Key Categories</div>
        <div className="grid grid-cols-1 gap-2.5 sm:grid-cols-3">
          {[
            {
              key: "play_modifiers" as const,
              label: "Modifier Keys",
              desc: "Shift, Cmd, Ctrl, Opt",
            },
            {
              key: "play_function_keys" as const,
              label: "Function Keys",
              desc: "F1 – F12 row",
            },
            {
              key: "play_numpad" as const,
              label: "Numpad Keys",
              desc: "Right-panned cluster",
            },
          ].map((item) => (
            <div
              key={item.key}
              className="flex items-center justify-between rounded-xl border border-zinc-800/80 bg-zinc-950/60 px-3 py-2.5"
            >
              <div>
                <div className="text-xs font-medium text-zinc-100">{item.label}</div>
                <div className="text-[10px] text-zinc-500">{item.desc}</div>
              </div>
              <ToggleSwitch
                checked={config.key_categories[item.key]}
                onChange={(val) =>
                  patchConfig((c) => ({
                    ...c,
                    key_categories: { ...c.key_categories, [item.key]: val },
                  }))
                }
              />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
