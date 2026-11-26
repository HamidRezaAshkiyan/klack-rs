import { create } from "zustand";
import { api } from "../lib/tauri";
import type {
  AppConfig,
  AudioDeviceInfo,
  PermissionStatus,
  SettingsTabId,
  SleepStatus,
  SpriteInfo,
  SwitchPack,
  TypingStats,
} from "../types";

interface AppStore {
  loaded: boolean;
  uiMode: "settings" | "popover";
  activeTab: SettingsTabId;
  config: AppConfig | null;
  switchPacks: SwitchPack[];
  sprites: SpriteInfo[];
  audioDevices: AudioDeviceInfo[];
  sleepStatus: SleepStatus;
  stats: TypingStats | null;
  permissions: PermissionStatus | null;

  bootstrap: () => Promise<void>;
  setUiMode: (mode: "settings" | "popover") => void;
  setActiveTab: (tab: SettingsTabId) => void;
  patchConfig: (updater: (prev: AppConfig) => AppConfig) => Promise<void>;
  toggleEnabled: () => Promise<void>;
  refreshAudioDevices: () => Promise<void>;
  refreshPermissions: () => Promise<void>;
  saveCustomPack: (pack: SwitchPack) => Promise<void>;
  deleteCustomPack: (packId: string) => Promise<void>;
  resetStats: () => Promise<void>;

  // Event sync setters
  setEnabledFromEvent: (enabled: boolean) => void;
  setConfigFromEvent: (config: AppConfig) => void;
  setSleepStatus: (status: SleepStatus) => void;
  setStats: (stats: TypingStats) => void;
}

export const useAppStore = create<AppStore>((set, get) => ({
  loaded: false,
  uiMode: "settings",
  activeTab: "switches",
  config: null,
  switchPacks: [],
  sprites: [],
  audioDevices: [],
  sleepStatus: { is_sleeping: false, reason: null, active_app: null },
  stats: null,
  permissions: null,

  bootstrap: async () => {
    const data = await api.getBootstrap();
    set({
      loaded: true,
      config: data.config,
      switchPacks: data.switch_packs,
      sprites: data.sprites,
      audioDevices: data.audio_devices,
      sleepStatus: data.sleep_status,
      stats: data.stats,
      permissions: data.permissions,
    });
  },

  setUiMode: (uiMode) => set({ uiMode }),
  setActiveTab: (activeTab) => set({ activeTab }),

  patchConfig: async (updater) => {
    const prev = get().config;
    if (!prev) return;
    const next = updater(prev);
    set({ config: next });
    try {
      const saved = await api.updateConfig(next);
      set({ config: saved });
    } catch (err) {
      console.error("Failed to update config:", err);
      set({ config: prev });
    }
  },

  toggleEnabled: async () => {
    const next = await api.toggleEnabled();
    const cur = get().config;
    if (cur) set({ config: { ...cur, enabled: next } });
  },

  refreshAudioDevices: async () => {
    const audioDevices = await api.listAudioDevices();
    set({ audioDevices });
  },

  refreshPermissions: async () => {
    const permissions = await api.getPermissions();
    set({ permissions });
  },

  saveCustomPack: async (pack) => {
    const switchPacks = await api.saveCustomPack(pack);
    const cur = get().config;
    set({
      switchPacks,
      config: cur ? { ...cur, active_switch_id: pack.id } : cur,
    });
  },

  deleteCustomPack: async (packId) => {
    const switchPacks = await api.deleteCustomPack(packId);
    const bootstrap = await api.getBootstrap();
    set({ switchPacks, config: bootstrap.config });
  },

  resetStats: async () => {
    const stats = await api.resetStats();
    set({ stats });
  },

  setEnabledFromEvent: (enabled) => {
    const cur = get().config;
    if (cur) set({ config: { ...cur, enabled } });
  },

  setConfigFromEvent: (config) => set({ config }),
  setSleepStatus: (sleepStatus) => set({ sleepStatus }),
  setStats: (stats) => set({ stats }),
}));
