import { useState } from "react";
import { Check, Play, Plus, Sparkles, Trash2 } from "lucide-react";
import clsx from "clsx";
import { api } from "../lib/tauri";
import { useAppStore } from "../store/useAppStore";
import type { SwitchPack } from "../types";

export function SwitchesTab() {
  const { config, switchPacks, sprites, patchConfig, saveCustomPack, deleteCustomPack } =
    useAppStore();
  const [showBuilder, setShowBuilder] = useState(false);
  const [name, setName] = useState("My Custom Board");
  const [feel, setFeel] = useState("Custom Mix");
  const [alpha, setAlpha] = useState("hi75-soft");
  const [space, setSpace] = useState("halo-space");
  const [enter, setEnter] = useState("uhk-medium");
  const [backspace, setBackspace] = useState("hi75-medium");
  const [modifier, setModifier] = useState("halo-shift");

  if (!config) return null;

  const handleCreateCustom = async () => {
    const id = `custom-${Date.now()}`;
    const pack: SwitchPack = {
      id,
      name: name.trim() || "Custom Switch Set",
      brand: "Custom Mix",
      feel: feel.trim() || "Tactile",
      description: "User-crafted switch set combining button-sounds sprites per key zone.",
      accent_color: "#f59e0b",
      is_custom: true,
      zones: {
        alpha: { sprite: alpha, pitch_scale: 1.0, gain_scale: 1.0 },
        space: { sprite: space, pitch_scale: 1.0, gain_scale: 1.08 },
        enter: { sprite: enter, pitch_scale: 0.97, gain_scale: 1.04 },
        backspace: { sprite: backspace, pitch_scale: 1.0, gain_scale: 1.02 },
        modifier: { sprite: modifier, pitch_scale: 0.96, gain_scale: 0.95 },
        function: { sprite: alpha, pitch_scale: 1.06, gain_scale: 0.92 },
        numpad: { sprite: alpha, pitch_scale: 1.0, gain_scale: 0.96 },
      },
    };
    await saveCustomPack(pack);
    setShowBuilder(false);
    api.previewSwitch(pack.id);
  };

  return (
    <div className="space-y-5">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold tracking-tight">Switch Sets</h2>
          <p className="text-xs text-zinc-400">
            Multi-sampled up/down recordings powered by{" "}
            <span className="text-zinc-200">joshwcomeau/button-sounds</span> (MIT).
          </p>
        </div>
        <button
          type="button"
          onClick={() => setShowBuilder((v) => !v)}
          className="inline-flex items-center gap-1.5 rounded-xl border border-amber-500/40 bg-amber-500/15 px-3 py-1.5 text-xs font-medium text-amber-300 hover:bg-amber-500/25"
        >
          <Plus className="h-3.5 w-3.5" />
          Mix &amp; Match Builder
        </button>
      </div>

      {/* Custom Mix & Match Builder */}
      {showBuilder && (
        <div className="rounded-2xl border border-amber-500/30 bg-zinc-900/80 p-4 space-y-3">
          <div className="flex items-center gap-2 text-xs font-semibold text-amber-300">
            <Sparkles className="h-4 w-4" />
            Create Custom Switch Set (55 Sprites Available)
          </div>
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="mb-1 block text-[11px] text-zinc-400">Name</label>
              <input
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-2.5 py-1.5 text-xs text-zinc-100 focus:border-amber-500 focus:outline-none"
              />
            </div>
            <div>
              <label className="mb-1 block text-[11px] text-zinc-400">Feel Tag</label>
              <input
                type="text"
                value={feel}
                onChange={(e) => setFeel(e.target.value)}
                className="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-2.5 py-1.5 text-xs text-zinc-100 focus:border-amber-500 focus:outline-none"
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-3 sm:grid-cols-3">
            {[
              { label: "Alpha Keys", value: alpha, set: setAlpha },
              { label: "Spacebar", value: space, set: setSpace },
              { label: "Enter / Return", value: enter, set: setEnter },
              { label: "Backspace", value: backspace, set: setBackspace },
              { label: "Modifiers", value: modifier, set: setModifier },
            ].map((row) => (
              <div key={row.label}>
                <label className="mb-1 block text-[11px] text-zinc-400">
                  {row.label}
                </label>
                <div className="flex gap-1">
                  <select
                    value={row.value}
                    onChange={(e) => row.set(e.target.value)}
                    className="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-2 py-1.5 text-xs text-zinc-100 focus:border-amber-500 focus:outline-none"
                  >
                    {sprites.map((s) => (
                      <option key={s.id} value={s.id}>
                        {s.label} ({s.category})
                      </option>
                    ))}
                  </select>
                  <button
                    type="button"
                    onClick={() => api.previewSprite(row.value)}
                    className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-zinc-700 bg-zinc-800 text-zinc-300 hover:text-amber-300"
                    title="Preview sprite"
                  >
                    <Play className="h-3 w-3 fill-current" />
                  </button>
                </div>
              </div>
            ))}
          </div>

          <div className="flex justify-end gap-2 pt-1">
            <button
              type="button"
              onClick={() => setShowBuilder(false)}
              className="rounded-lg border border-zinc-700 px-3 py-1.5 text-xs text-zinc-300 hover:bg-zinc-800"
            >
              Cancel
            </button>
            <button
              type="button"
              onClick={handleCreateCustom}
              className="rounded-lg bg-amber-500 px-3.5 py-1.5 text-xs font-semibold text-zinc-950 hover:bg-amber-400"
            >
              Save &amp; Activate
            </button>
          </div>
        </div>
      )}

      {/* Switch Cards Grid */}
      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
        {switchPacks.map((pack) => {
          const active = pack.id === config.active_switch_id;
          return (
            <div
              key={pack.id}
              onClick={() =>
                patchConfig((c) => ({ ...c, active_switch_id: pack.id }))
              }
              className={clsx(
                "group relative cursor-pointer rounded-2xl border p-4 transition-all duration-150",
                active
                  ? "border-amber-500/60 bg-gradient-to-br from-amber-500/12 to-zinc-900/90 shadow-lg shadow-amber-500/5"
                  : "border-zinc-800/80 bg-zinc-900/50 hover:border-zinc-700 hover:bg-zinc-900/80",
              )}
            >
              <div className="flex items-start justify-between gap-2">
                <div className="flex items-center gap-2.5">
                  <div
                    className="flex h-9 w-9 items-center justify-center rounded-xl border border-white/10 shadow-inner"
                    style={{
                      backgroundColor: `${pack.accent_color}22`,
                      color: pack.accent_color,
                    }}
                  >
                    <span
                      className="h-3 w-3 rounded-sm"
                      style={{ backgroundColor: pack.accent_color }}
                    />
                  </div>
                  <div>
                    <div className="flex items-center gap-1.5">
                      <span className="text-sm font-semibold text-zinc-100">
                        {pack.name}
                      </span>
                      {active && (
                        <span className="inline-flex items-center gap-0.5 rounded-full bg-amber-500/20 px-1.5 py-0.5 text-[10px] font-medium text-amber-300">
                          <Check className="h-2.5 w-2.5" /> Active
                        </span>
                      )}
                    </div>
                    <div className="text-[11px] text-zinc-400">
                      {pack.brand} · {pack.feel}
                    </div>
                  </div>
                </div>

                <div className="flex items-center gap-1.5">
                  {pack.is_custom && (
                    <button
                      type="button"
                      title="Delete custom switch set"
                      onClick={(e) => {
                        e.stopPropagation();
                        deleteCustomPack(pack.id);
                      }}
                      className="flex h-8 w-8 items-center justify-center rounded-lg border border-zinc-800 bg-zinc-900 text-zinc-400 hover:border-red-500/50 hover:text-red-400"
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                    </button>
                  )}
                  <button
                    type="button"
                    onClick={(e) => {
                      e.stopPropagation();
                      api.previewSwitch(pack.id);
                    }}
                    className="inline-flex items-center gap-1 rounded-lg border border-zinc-700/80 bg-zinc-800/80 px-2.5 py-1.5 text-[11px] font-medium text-zinc-200 hover:border-amber-500/50 hover:text-amber-300"
                  >
                    <Play className="h-3 w-3 fill-current" />
                    Preview
                  </button>
                </div>
              </div>

              <p className="mt-2.5 text-xs leading-relaxed text-zinc-400">
                {pack.description}
              </p>

              <div className="mt-3 flex flex-wrap gap-1.5 text-[10px] text-zinc-500">
                <span className="rounded-md bg-zinc-800/80 px-1.5 py-0.5">
                  Alpha: {pack.zones.alpha.sprite}
                </span>
                <span className="rounded-md bg-zinc-800/80 px-1.5 py-0.5">
                  Space: {pack.zones.space.sprite}
                </span>
                <span className="rounded-md bg-zinc-800/80 px-1.5 py-0.5">
                  Enter: {pack.zones.enter.sprite}
                </span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
