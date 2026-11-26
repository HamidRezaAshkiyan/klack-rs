import { Activity, Flame, Keyboard, Mouse, RotateCcw, Waves } from "lucide-react";
import { useAppStore } from "../store/useAppStore";

export function StatsTab() {
  const { stats, resetStats } = useAppStore();
  if (!stats) return null;

  const zoneEntries = Object.entries(stats.zone_counts).sort((a, b) => b[1] - a[1]);
  const maxZone = Math.max(1, ...zoneEntries.map(([, v]) => v));

  return (
    <div className="space-y-5">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold tracking-tight">Typing Stats</h2>
          <p className="text-xs text-zinc-400">
            100% local counters — keystroke text is never recorded or stored.
          </p>
        </div>
        <button
          type="button"
          onClick={() => resetStats()}
          className="inline-flex items-center gap-1.5 rounded-xl border border-zinc-700 bg-zinc-900 px-3 py-1.5 text-xs text-zinc-300 hover:border-red-500/40 hover:text-red-300"
        >
          <RotateCcw className="h-3.5 w-3.5" />
          Reset Stats
        </button>
      </div>

      {/* Primary KPI Grid */}
      <div className="grid grid-cols-2 gap-3 sm:grid-cols-3">
        <div className="rounded-2xl border border-zinc-800/80 bg-zinc-900/50 p-4">
          <div className="flex items-center gap-1.5 text-[11px] text-zinc-400">
            <Keyboard className="h-3.5 w-3.5 text-amber-400" />
            Keystrokes Today
          </div>
          <div className="mt-1.5 font-mono text-2xl font-bold text-zinc-100">
            {stats.keystrokes_today.toLocaleString()}
          </div>
          <div className="mt-1 text-[10px] text-zinc-500">
            All-time: {stats.keystrokes_all_time.toLocaleString()}
          </div>
        </div>

        <div className="rounded-2xl border border-zinc-800/80 bg-zinc-900/50 p-4">
          <div className="flex items-center gap-1.5 text-[11px] text-zinc-400">
            <Activity className="h-3.5 w-3.5 text-emerald-400" />
            Live Speed
          </div>
          <div className="mt-1.5 font-mono text-2xl font-bold text-zinc-100">
            {stats.current_wpm} <span className="text-xs font-normal text-zinc-400">WPM</span>
          </div>
          <div className="mt-1 flex items-center gap-1 text-[10px] text-zinc-500">
            <Flame className="h-3 w-3 text-amber-400" />
            Peak: {stats.peak_wpm} WPM
          </div>
        </div>

        <div className="rounded-2xl border border-zinc-800/80 bg-zinc-900/50 p-4 col-span-2 sm:col-span-1">
          <div className="flex items-center gap-1.5 text-[11px] text-zinc-400">
            <Mouse className="h-3.5 w-3.5 text-sky-400" />
            Mouse &amp; Scroll
          </div>
          <div className="mt-1.5 font-mono text-xl font-bold text-zinc-100">
            {stats.mouse_clicks_today.toLocaleString()}{" "}
            <span className="text-xs font-normal text-zinc-400">clicks</span>
          </div>
          <div className="mt-1 flex items-center gap-1 text-[10px] text-zinc-500">
            <Waves className="h-3 w-3 text-sky-400" />
            {stats.scroll_ticks_today.toLocaleString()} scroll ticks today
          </div>
        </div>
      </div>

      {/* Key Zone Breakdown */}
      <div className="rounded-2xl border border-zinc-800/80 bg-zinc-900/50 p-4">
        <div className="mb-3 text-xs font-semibold text-zinc-200">Key Zone Activity</div>
        {zoneEntries.length === 0 ? (
          <p className="py-6 text-center text-xs text-zinc-500">
            Start typing anywhere to see live zone analytics.
          </p>
        ) : (
          <div className="space-y-2.5">
            {zoneEntries.map(([zone, count]) => (
              <div key={zone} className="space-y-1">
                <div className="flex justify-between text-[11px]">
                  <span className="capitalize text-zinc-300">{zone}</span>
                  <span className="font-mono text-zinc-400">{count.toLocaleString()}</span>
                </div>
                <div className="h-2 w-full overflow-hidden rounded-full bg-zinc-800">
                  <div
                    className="h-full rounded-full bg-gradient-to-r from-amber-500 to-amber-300 transition-all duration-300"
                    style={{ width: `${Math.max(4, (count / maxZone) * 100)}%` }}
                  />
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
