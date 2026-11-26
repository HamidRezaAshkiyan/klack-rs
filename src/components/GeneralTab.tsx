import { useState } from "react";
import {
  Bell,
  CheckCircle2,
  EyeOff,
  Keyboard,
  Power,
  RefreshCw,
  ShieldAlert,
} from "lucide-react";
import { api } from "../lib/tauri";
import { useAppStore } from "../store/useAppStore";
import { ToggleSwitch } from "./ToggleSwitch";

export function GeneralTab() {
  const { config, permissions, patchConfig, refreshPermissions } = useAppStore();
  const [shortcutDraft, setShortcutDraft] = useState(config?.toggle_shortcut ?? "");

  if (!config) return null;

  const saveShortcut = () => {
    patchConfig((c) => ({ ...c, toggle_shortcut: shortcutDraft.trim() }));
  };

  return (
    <div className="space-y-5">
      <div>
        <h2 className="text-lg font-semibold tracking-tight">General &amp; System</h2>
        <p className="text-xs text-zinc-400">
          Global hotkey, startup behavior, menu bar visibility, and OS permissions.
        </p>
      </div>

      {/* Accessibility Permission Banner */}
      {permissions && (
        <div
          className={`rounded-2xl border p-4 ${
            permissions.accessibility_granted
              ? "border-emerald-500/30 bg-emerald-500/5"
              : "border-amber-500/40 bg-amber-500/10"
          }`}
        >
          <div className="flex items-start justify-between gap-3">
            <div className="flex items-start gap-3">
              {permissions.accessibility_granted ? (
                <CheckCircle2 className="mt-0.5 h-5 w-5 shrink-0 text-emerald-400" />
              ) : (
                <ShieldAlert className="mt-0.5 h-5 w-5 shrink-0 text-amber-400" />
              )}
              <div>
                <div className="text-xs font-semibold text-zinc-100">
                  {permissions.accessibility_granted
                    ? "Global Input Capture Active"
                    : "Accessibility Permission Required"}
                </div>
                <p className="mt-0.5 text-[11px] leading-relaxed text-zinc-400">
                  {permissions.note}
                </p>
              </div>
            </div>
            <div className="flex shrink-0 items-center gap-1.5">
              <button
                type="button"
                onClick={() => refreshPermissions()}
                className="flex h-8 w-8 items-center justify-center rounded-lg border border-zinc-700 bg-zinc-900 text-zinc-300 hover:text-white"
                title="Re-check permission"
              >
                <RefreshCw className="h-3.5 w-3.5" />
              </button>
              {!permissions.accessibility_granted && permissions.platform === "macos" && (
                <button
                  type="button"
                  onClick={() => api.openAccessibilitySettings()}
                  className="rounded-lg bg-amber-500 px-3 py-1.5 text-xs font-semibold text-zinc-950 hover:bg-amber-400"
                >
                  Open System Settings
                </button>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Global Shortcut */}
      <div className="rounded-2xl border border-zinc-800/80 bg-zinc-900/50 p-4 space-y-2.5">
        <div className="flex items-center gap-2 text-xs font-semibold text-zinc-100">
          <Keyboard className="h-4 w-4 text-amber-400" />
          Global Toggle Shortcut
        </div>
        <p className="text-[11px] text-zinc-400">
          Toggle Klack-rs sound on or off from any application (e.g.{" "}
          <code className="rounded bg-zinc-800 px-1 py-0.5 text-zinc-200">Alt+Super+K</code>{" "}
          on macOS or{" "}
          <code className="rounded bg-zinc-800 px-1 py-0.5 text-zinc-200">Ctrl+Alt+K</code>{" "}
          on Windows/Linux).
        </p>
        <div className="flex gap-2">
          <input
            type="text"
            value={shortcutDraft}
            onChange={(e) => setShortcutDraft(e.target.value)}
            className="flex-1 rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-1.5 font-mono text-xs text-zinc-100 focus:border-amber-500 focus:outline-none"
          />
          <button
            type="button"
            onClick={saveShortcut}
            className="rounded-xl bg-amber-500 px-3.5 py-1.5 text-xs font-semibold text-zinc-950 hover:bg-amber-400"
          >
            Apply
          </button>
        </div>
      </div>

      {/* System Toggles */}
      <div className="space-y-2">
        <div className="flex items-center justify-between rounded-2xl border border-zinc-800/80 bg-zinc-900/50 px-4 py-3">
          <div className="flex items-center gap-3">
            <Power className="h-4 w-4 text-zinc-400" />
            <div>
              <div className="text-xs font-semibold text-zinc-100">Launch at Login</div>
              <div className="text-[11px] text-zinc-400">
                Start Klack-rs automatically when you sign in
              </div>
            </div>
          </div>
          <ToggleSwitch
            checked={config.launch_at_login}
            onChange={(launch_at_login) =>
              patchConfig((c) => ({ ...c, launch_at_login }))
            }
          />
        </div>

        <div className="flex items-center justify-between rounded-2xl border border-zinc-800/80 bg-zinc-900/50 px-4 py-3">
          <div className="flex items-center gap-3">
            <Bell className="h-4 w-4 text-zinc-400" />
            <div>
              <div className="text-xs font-semibold text-zinc-100">
                Switch Change Notifications
              </div>
              <div className="text-[11px] text-zinc-400">
                Show a subtle OS notification when hot-swapping switch sets
              </div>
            </div>
          </div>
          <ToggleSwitch
            checked={config.notify_on_switch_change}
            onChange={(notify_on_switch_change) =>
              patchConfig((c) => ({ ...c, notify_on_switch_change }))
            }
          />
        </div>

        <div className="flex items-center justify-between rounded-2xl border border-zinc-800/80 bg-zinc-900/50 px-4 py-3">
          <div className="flex items-center gap-3">
            <EyeOff className="h-4 w-4 text-zinc-400" />
            <div>
              <div className="text-xs font-semibold text-zinc-100">
                Hide Menu Bar / Tray Icon
              </div>
              <div className="text-[11px] text-zinc-400">
                Run invisibly in the background (use global shortcut to toggle)
              </div>
            </div>
          </div>
          <ToggleSwitch
            checked={config.hide_tray_icon}
            onChange={(hide_tray_icon) =>
              patchConfig((c) => ({ ...c, hide_tray_icon }))
            }
          />
        </div>
      </div>

      {/* Attribution */}
      <div className="rounded-2xl border border-zinc-800/60 bg-zinc-950/60 p-4 text-[11px] text-zinc-500">
        <strong className="text-zinc-300">Klack-rs</strong> · Built with Rust, Tauri v2, cpal, and
        React 19. Bundled multi-sampled switch &amp; mouse recordings from{" "}
        <span className="text-zinc-300">joshwcomeau/button-sounds</span> (MIT License).
      </div>
    </div>
  );
}
