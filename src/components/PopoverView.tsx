import { useState } from "react";
import {
  Keyboard,
  Moon,
  Mouse,
  Play,
  Search,
  SlidersHorizontal,
  Volume2,
  VolumeX,
  Waves,
} from "lucide-react";
import clsx from "clsx";
import { api } from "../lib/tauri";
import { useAppStore } from "../store/useAppStore";
import { ToggleSwitch } from "./ToggleSwitch";

export function PopoverView() {
  const { config, switchPacks, sleepStatus, patchConfig, toggleEnabled, setUiMode } =
    useAppStore();
  const [query, setQuery] = useState("");

  if (!config) return null;

  const q = query.trim().toLowerCase();
  const visiblePacks = q
    ? switchPacks.filter(
        (p) =>
          p.name.toLowerCase().includes(q) ||
          p.category.toLowerCase().includes(q) ||
          p.feel.toLowerCase().includes(q),
      )
    : switchPacks;

  return (
    <div className="flex h-screen flex-col bg-zinc-950 p-4 text-zinc-100">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-zinc-800/80 pb-3">
        <div className="flex items-center gap-2.5">
          <div
            className={clsx(
              "flex h-8 w-8 items-center justify-center rounded-lg border",
              config.enabled && !sleepStatus.is_sleeping
                ? "border-amber-500/40 bg-amber-500/15 text-amber-400"
                : sleepStatus.is_sleeping
                  ? "border-indigo-500/40 bg-indigo-500/15 text-indigo-400"
                  : "border-zinc-700 bg-zinc-900 text-zinc-500",
            )}
          >
            <Keyboard className="h-4 w-4" />
          </div>
          <div>
            <h1 className="text-sm font-semibold tracking-tight">Klack-rs</h1>
            <p className="text-[11px] text-zinc-400">
              {!config.enabled
                ? "Muted"
                : sleepStatus.is_sleeping
                  ? sleepStatus.reason ?? "Sleeping"
                  : "Active · Sub-5ms Native"}
            </p>
          </div>
        </div>
        <ToggleSwitch checked={config.enabled} onChange={() => toggleEnabled()} />
      </div>

      {/* Volume */}
      <div className="mt-3 rounded-xl border border-zinc-800/80 bg-zinc-900/60 p-3">
        <div className="mb-1.5 flex items-center justify-between text-xs">
          <span className="flex items-center gap-1.5 text-zinc-300">
            {config.volume === 0 || !config.enabled ? (
              <VolumeX className="h-3.5 w-3.5 text-zinc-500" />
            ) : (
              <Volume2 className="h-3.5 w-3.5 text-amber-400" />
            )}
            Volume
          </span>
          <span className="font-mono text-[11px] text-zinc-400">
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

      {/* Switches Quick Picker */}
      <div className="mt-3 flex min-h-0 flex-1 flex-col">
        <div className="mb-1.5 flex items-center justify-between px-1">
          <span className="text-[11px] font-medium uppercase tracking-wider text-zinc-500">
            Presets ({visiblePacks.length})
          </span>
        </div>
        <div className="relative mb-2">
          <Search className="pointer-events-none absolute left-2.5 top-1/2 h-3 w-3 -translate-y-1/2 text-zinc-500" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Filter presets..."
            className="w-full rounded-lg border border-zinc-800 bg-zinc-900/70 py-1 pl-7 pr-2 text-[11px] text-zinc-100 placeholder:text-zinc-500 focus:border-amber-500/50 focus:outline-none"
          />
        </div>
        <div className="flex-1 space-y-1.5 overflow-y-auto pr-0.5">
          {visiblePacks.map((pack) => {
            const active = pack.id === config.active_switch_id;
            return (
              <div
                key={pack.id}
                onClick={() =>
                  patchConfig((c) => ({ ...c, active_switch_id: pack.id }))
                }
                className={clsx(
                  "flex cursor-pointer items-center justify-between rounded-xl border px-3 py-2 transition-all",
                  active
                    ? "border-amber-500/50 bg-amber-500/10"
                    : "border-zinc-800/70 bg-zinc-900/40 hover:border-zinc-700 hover:bg-zinc-900",
                )}
              >
                <div className="flex items-center gap-2.5 min-w-0">
                  <span
                    className="h-2.5 w-2.5 shrink-0 rounded-full"
                    style={{ backgroundColor: pack.accent_color }}
                  />
                  <div className="truncate">
                    <div className="truncate text-xs font-medium text-zinc-100">
                      {pack.name}
                    </div>
                    <div className="truncate text-[10px] text-zinc-400">
                      {pack.brand} · {pack.feel}
                    </div>
                  </div>
                </div>
                <button
                  type="button"
                  title="Preview switch"
                  onClick={(e) => {
                    e.stopPropagation();
                    api.previewSwitch(pack.id);
                  }}
                  className="ml-2 flex h-7 w-7 shrink-0 items-center justify-center rounded-lg border border-zinc-700/80 bg-zinc-800/80 text-zinc-300 hover:border-amber-500/50 hover:text-amber-300"
                >
                  <Play className="h-3 w-3 fill-current" />
                </button>
              </div>
            );
          })}
        </div>
      </div>

      {/* Quick Toggles */}
      <div className="mt-3 grid grid-cols-3 gap-1.5 border-t border-zinc-800/80 pt-3">
        <button
          type="button"
          onClick={() =>
            patchConfig((c) => ({
              ...c,
              mouse: { ...c.mouse, enabled: !c.mouse.enabled },
            }))
          }
          className={clsx(
            "flex items-center justify-center gap-1.5 rounded-lg border px-2 py-1.5 text-[11px] font-medium transition",
            config.mouse.enabled
              ? "border-amber-500/40 bg-amber-500/15 text-amber-300"
              : "border-zinc-800 bg-zinc-900/60 text-zinc-400 hover:text-zinc-200",
          )}
        >
          <Mouse className="h-3.5 w-3.5" />
          Mouse
        </button>
        <button
          type="button"
          onClick={() =>
            patchConfig((c) => ({
              ...c,
              scroll: { ...c.scroll, enabled: !c.scroll.enabled },
            }))
          }
          className={clsx(
            "flex items-center justify-center gap-1.5 rounded-lg border px-2 py-1.5 text-[11px] font-medium transition",
            config.scroll.enabled
              ? "border-amber-500/40 bg-amber-500/15 text-amber-300"
              : "border-zinc-800 bg-zinc-900/60 text-zinc-400 hover:text-zinc-200",
          )}
        >
          <Waves className="h-3.5 w-3.5" />
          Scroll
        </button>
        <button
          type="button"
          onClick={() =>
            patchConfig((c) => ({
              ...c,
              sleep_triggers: {
                ...c.sleep_triggers,
                manual_sleep: !c.sleep_triggers.manual_sleep,
              },
            }))
          }
          className={clsx(
            "flex items-center justify-center gap-1.5 rounded-lg border px-2 py-1.5 text-[11px] font-medium transition",
            config.sleep_triggers.manual_sleep
              ? "border-indigo-500/40 bg-indigo-500/15 text-indigo-300"
              : "border-zinc-800 bg-zinc-900/60 text-zinc-400 hover:text-zinc-200",
          )}
        >
          <Moon className="h-3.5 w-3.5" />
          Sleep
        </button>
      </div>

      {/* Footer */}
      <button
        type="button"
        onClick={() => setUiMode("settings")}
        className="mt-2.5 flex w-full items-center justify-center gap-2 rounded-xl border border-zinc-800 bg-zinc-900 py-2 text-xs font-medium text-zinc-200 hover:border-zinc-700 hover:bg-zinc-800/90"
      >
        <SlidersHorizontal className="h-3.5 w-3.5" />
        Klack Settings...
      </button>
    </div>
  );
}
