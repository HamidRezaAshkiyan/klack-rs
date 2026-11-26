import { useState } from "react";
import { BellOff, Gamepad2, MonitorUp, Moon, Music, Plus, X } from "lucide-react";
import clsx from "clsx";
import { useAppStore } from "../store/useAppStore";
import { ToggleSwitch } from "./ToggleSwitch";

export function SleepTab() {
  const { config, sleepStatus, patchConfig } = useAppStore();
  const [newApp, setNewApp] = useState("");

  if (!config) return null;

  const addExcludedApp = () => {
    const trimmed = newApp.trim();
    if (!trimmed) return;
    if (config.sleep_triggers.excluded_apps.includes(trimmed)) {
      setNewApp("");
      return;
    }
    patchConfig((c) => ({
      ...c,
      sleep_triggers: {
        ...c.sleep_triggers,
        excluded_apps: [...c.sleep_triggers.excluded_apps, trimmed],
      },
    }));
    setNewApp("");
  };

  const removeExcludedApp = (appName: string) => {
    patchConfig((c) => ({
      ...c,
      sleep_triggers: {
        ...c.sleep_triggers,
        excluded_apps: c.sleep_triggers.excluded_apps.filter((a) => a !== appName),
      },
    }));
  };

  const triggers = [
    {
      key: "manual_sleep" as const,
      icon: Moon,
      title: "Manual Sleep",
      desc: "Pause all sounds until you wake Klack-rs back up.",
    },
    {
      key: "on_screen_sharing" as const,
      icon: MonitorUp,
      title: "Screen Sharing & Calls",
      desc: "Auto-pause during Zoom, Discord, OBS, Loom, Teams, or FaceTime.",
    },
    {
      key: "on_gaming_fullscreen" as const,
      icon: Gamepad2,
      title: "Gaming / Fullscreen",
      desc: "Auto-pause when a fullscreen game or app is in focus.",
    },
    {
      key: "on_focus_dnd" as const,
      icon: BellOff,
      title: "Focus / Do Not Disturb",
      desc: "Respect OS Focus mode and silence keystrokes automatically.",
    },
    {
      key: "on_media_playing" as const,
      icon: Music,
      title: "Media Playback",
      desc: "Pause keyboard sounds while system audio/media is playing.",
    },
  ];

  return (
    <div className="space-y-5">
      <div>
        <h2 className="text-lg font-semibold tracking-tight">Sleep Triggers</h2>
        <p className="text-xs text-zinc-400">
          Automatically silence Klack-rs during calls, gaming sessions, Focus mode, or specific apps.
        </p>
      </div>

      {/* Live Status Banner */}
      <div
        className={clsx(
          "flex items-center justify-between rounded-2xl border p-4",
          sleepStatus.is_sleeping
            ? "border-indigo-500/40 bg-indigo-500/10"
            : "border-emerald-500/30 bg-emerald-500/5",
        )}
      >
        <div className="flex items-center gap-3">
          <div
            className={clsx(
              "h-2.5 w-2.5 rounded-full",
              sleepStatus.is_sleeping ? "bg-indigo-400 animate-pulse" : "bg-emerald-400",
            )}
          />
          <div>
            <div className="text-xs font-semibold text-zinc-100">
              {sleepStatus.is_sleeping ? "Currently Sleeping" : "Awake & Listening"}
            </div>
            <div className="text-[11px] text-zinc-400">
              {sleepStatus.is_sleeping
                ? sleepStatus.reason ?? "Sleep trigger active"
                : sleepStatus.active_app
                  ? `Active app: ${sleepStatus.active_app}`
                  : "No sleep triggers active"}
            </div>
          </div>
        </div>
      </div>

      {/* Trigger List */}
      <div className="space-y-2">
        {triggers.map((t) => {
          const Icon = t.icon;
          return (
            <div
              key={t.key}
              className="flex items-center justify-between rounded-2xl border border-zinc-800/80 bg-zinc-900/50 px-4 py-3"
            >
              <div className="flex items-center gap-3">
                <div className="flex h-8 w-8 items-center justify-center rounded-xl bg-zinc-800/80 text-zinc-300">
                  <Icon className="h-4 w-4" />
                </div>
                <div>
                  <div className="text-xs font-semibold text-zinc-100">{t.title}</div>
                  <div className="text-[11px] text-zinc-400">{t.desc}</div>
                </div>
              </div>
              <ToggleSwitch
                accent="indigo"
                checked={config.sleep_triggers[t.key]}
                onChange={(val) =>
                  patchConfig((c) => ({
                    ...c,
                    sleep_triggers: { ...c.sleep_triggers, [t.key]: val },
                  }))
                }
              />
            </div>
          );
        })}
      </div>

      {/* Excluded Apps */}
      <div className="rounded-2xl border border-zinc-800/80 bg-zinc-900/50 p-4 space-y-3">
        <div>
          <div className="text-xs font-semibold text-zinc-100">Excluded Apps</div>
          <div className="text-[11px] text-zinc-400">
            Klack-rs automatically sleeps whenever any of these apps is frontmost.
          </div>
        </div>

        <div className="flex gap-2">
          <input
            type="text"
            value={newApp}
            placeholder="e.g. Slack, OBS, Logic Pro..."
            onChange={(e) => setNewApp(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") addExcludedApp();
            }}
            className="flex-1 rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-1.5 text-xs text-zinc-100 placeholder:text-zinc-500 focus:border-indigo-400 focus:outline-none"
          />
          <button
            type="button"
            onClick={addExcludedApp}
            className="inline-flex items-center gap-1 rounded-xl bg-zinc-800 px-3 py-1.5 text-xs font-medium text-zinc-100 hover:bg-zinc-700"
          >
            <Plus className="h-3.5 w-3.5" />
            Add
          </button>
        </div>

        <div className="flex flex-wrap gap-1.5">
          {config.sleep_triggers.excluded_apps.map((app) => (
            <span
              key={app}
              className="inline-flex items-center gap-1.5 rounded-lg border border-zinc-700/80 bg-zinc-950 px-2.5 py-1 text-xs text-zinc-200"
            >
              {app}
              <button
                type="button"
                onClick={() => removeExcludedApp(app)}
                className="text-zinc-500 hover:text-red-400"
              >
                <X className="h-3 w-3" />
              </button>
            </span>
          ))}
        </div>
      </div>
    </div>
  );
}
