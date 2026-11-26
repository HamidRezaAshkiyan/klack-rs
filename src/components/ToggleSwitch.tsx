import clsx from "clsx";

interface ToggleSwitchProps {
  checked: boolean;
  onChange: (next: boolean) => void;
  disabled?: boolean;
  accent?: "amber" | "indigo" | "emerald";
}

export function ToggleSwitch({
  checked,
  onChange,
  disabled = false,
  accent = "amber",
}: ToggleSwitchProps) {
  const bg = checked
    ? accent === "indigo"
      ? "bg-indigo-500"
      : accent === "emerald"
        ? "bg-emerald-500"
        : "bg-amber-500"
    : "bg-zinc-700/80";

  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={clsx(
        "relative inline-flex h-6 w-11 shrink-0 items-center rounded-full transition-colors duration-200 focus:outline-none focus-visible:ring-2 focus-visible:ring-amber-400/70",
        bg,
        disabled && "opacity-45 cursor-not-allowed",
      )}
    >
      <span
        className={clsx(
          "inline-block h-4.5 w-4.5 transform rounded-full bg-white shadow-sm transition-transform duration-200",
          checked ? "translate-x-[22px]" : "translate-x-[3px]",
        )}
      />
    </button>
  );
}
