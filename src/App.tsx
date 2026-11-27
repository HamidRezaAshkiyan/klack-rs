import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  BarChart3,
  Keyboard,
  Moon,
  Mouse,
  PanelTopClose,
  Settings,
  ShieldAlert,
  Sliders,
} from "lucide-react";
import clsx from "clsx";
import { api } from "./lib/tauri";
import { useAppStore } from "./store/useAppStore";
import { ToggleSwitch } from "./components/ToggleSwitch";
import { PopoverView } from "./components/PopoverView";
import { SwitchesTab } from "./components/SwitchesTab";
import { AudioToneTab } from "./components/AudioToneTab";
import { MouseScrollTab } from "./components/MouseScrollTab";
import { SleepTab } from "./components/SleepTab";
import { StatsTab } from "./components/StatsTab";
import { GeneralTab } from "./components/GeneralTab";
import type { AppConfig, SettingsTabId, SleepStatus, TypingStats } from "./types";

const NAV_ITEMS: Array<{ id: SettingsTabId; label: string; icon: typeof Keyboard }> = [
  { id: "switches", label: "Switches", icon: Keyboard },
  { id: "audio", label: "Audio & Tone", icon: Sliders },
  { id: "mouse", label: "Mouse & Scroll", icon: Mouse },
  { id: "sleep", label: "Sleep Triggers", icon: Moon },
  { id: "stats", label: "Stats", icon: BarChart3 },
  { id: "general", label: "General", icon: Settings },
];

export default function App() {
  const {
    loaded,
    uiMode,
    activeTab,
    config,
    sleepStatus,
    permissions,
    bootstrap,
    refreshPermissions,
    setUiMode,
    setActiveTab,
    toggleEnabled,
    setEnabledFromEvent,
    setConfigFromEvent,
    setSleepStatus,
    setStats,
  } = useAppStore();

  useEffect(() => {
    bootstrap();

    const unlisteners = [
      listen<boolean>("enabled-changed", (e) => setEnabledFromEvent(e.payload)),
      listen<AppConfig>("config-updated", (e) => setConfigFromEvent(e.payload)),
      listen<SleepStatus>("sleep-status-changed", (e) => setSleepStatus(e.payload)),
      listen<TypingStats>("stats-updated", (e) => setStats(e.payload)),
      listen<string>("ui-mode-changed", (e) => {
        if (e.payload === "popover" || e.payload === "settings") {
          setUiMode(e.payload);
        }
      }),
    ];

    const permTimer = window.setInterval(() => {
      refreshPermissions();
    }, 3000);

    return () => {
      window.clearInterval(permTimer);
      unlisteners.forEach((p) => p.then((fn) => fn()));
    };
  }, [
    bootstrap,
    refreshPermissions,
    setConfigFromEvent,
    setEnabledFromEvent,
    setSleepStatus,
    setStats,
    setUiMode,
  ]);

  if (!loaded || !config) {
    return (
      <div className="flex h-screen items-center justify-center bg-zinc-950 text-xs text-zinc-400">
        Loading Klack-rs audio engine...
      </div>
    );
  }

  if (uiMode === "popover") {
    return <PopoverView />;
  }

  return (
    <div className="flex h-screen bg-zinc-950 text-zinc-100">
      {/* Sidebar */}
      <aside className="flex w-52 shrink-0 flex-col border-r border-zinc-800/80 bg-zinc-950/90 p-3">
        <div className="flex items-center justify-between px-2 pb-4 pt-1">
          <div className="flex items-center gap-2.5">
            <div className="flex h-8 w-8 items-center justify-center rounded-xl border border-amber-500/40 bg-amber-500/15 text-amber-400">
              <Keyboard className="h-4 w-4" />
            </div>
            <div>
              <div className="text-sm font-bold tracking-tight">Klack-rs</div>
              <div className="text-[10px] text-zinc-500">v0.1.1 · Native</div>
            </div>
          </div>
        </div>

        {/* Master Sound Card */}
        <div className="mb-4 rounded-xl border border-zinc-800/80 bg-zinc-900/60 p-2.5">
          <div className="flex items-center justify-between">
            <span className="text-xs font-medium text-zinc-200">
              {!config.enabled
                ? "Muted"
                : sleepStatus.is_sleeping
                  ? "Sleeping"
                  : "Sound On"}
            </span>
            <ToggleSwitch checked={config.enabled} onChange={() => toggleEnabled()} />
          </div>
          {sleepStatus.is_sleeping && config.enabled && (
            <div className="mt-1.5 truncate text-[10px] text-indigo-300">
              {sleepStatus.reason}
            </div>
          )}
        </div>

        {/* Navigation */}
        <nav className="flex-1 space-y-1">
          {NAV_ITEMS.map((item) => {
            const Icon = item.icon;
            const active = activeTab === item.id;
            return (
              <button
                key={item.id}
                type="button"
                onClick={() => setActiveTab(item.id)}
                className={clsx(
                  "flex w-full items-center gap-2.5 rounded-xl px-3 py-2 text-xs font-medium transition-colors",
                  active
                    ? "bg-amber-500/15 text-amber-300"
                    : "text-zinc-400 hover:bg-zinc-900 hover:text-zinc-100",
                )}
              >
                <Icon className="h-4 w-4" />
                {item.label}
              </button>
            );
          })}
        </nav>

        {/* Compact Popover Toggle */}
        <button
          type="button"
          onClick={() => setUiMode("popover")}
          className="mt-2 flex items-center justify-center gap-1.5 rounded-xl border border-zinc-800 bg-zinc-900/60 px-3 py-2 text-[11px] font-medium text-zinc-400 hover:border-zinc-700 hover:text-zinc-200"
        >
          <PanelTopClose className="h-3.5 w-3.5" />
          Quick Menu View
        </button>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-y-auto p-6">
        <div className="mx-auto max-w-2xl space-y-4">
          {permissions && !permissions.accessibility_granted && (
            <div className="flex items-center justify-between gap-3 rounded-2xl border border-amber-500/50 bg-amber-500/15 px-4 py-3">
              <div className="flex items-center gap-2.5 min-w-0">
                <ShieldAlert className="h-5 w-5 shrink-0 text-amber-400" />
                <div className="text-xs text-amber-100">
                  <span className="font-semibold">Accessibility permission needed.</span>{" "}
                  <span className="text-amber-200/80">
                    After an update, remove &amp; re-add (or toggle off/on) Klack-rs in System Settings → Privacy &amp; Security → Accessibility.
                  </span>
                </div>
              </div>
              {permissions.platform === "macos" && (
                <button
                  type="button"
                  onClick={() => api.openAccessibilitySettings()}
                  className="shrink-0 rounded-lg bg-amber-500 px-3 py-1.5 text-xs font-semibold text-zinc-950 hover:bg-amber-400"
                >
                  Open Settings
                </button>
              )}
            </div>
          )}
          {activeTab === "switches" && <SwitchesTab />}
          {activeTab === "audio" && <AudioToneTab />}
          {activeTab === "mouse" && <MouseScrollTab />}
          {activeTab === "sleep" && <SleepTab />}
          {activeTab === "stats" && <StatsTab />}
          {activeTab === "general" && <GeneralTab />}
        </div>
      </main>
    </div>
  );
}
