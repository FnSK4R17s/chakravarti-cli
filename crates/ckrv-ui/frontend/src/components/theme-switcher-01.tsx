/**
 * @module ThemeSwitcher
 * @description
 * Toggle switch for light/dark mode with sun/moon icons.
 * Persists preference to localStorage and applies theme to document root.
 *
 * @context
 * Rendered in the app header/toolbar for global theme control.
 * Initializes from localStorage, then falls back to system preference.
 *
 * @dependencies
 * - lucide-react: Sun/Moon icons
 * - @/components/ui/switch: Base toggle component
 *
 * @example
 * // In header component
 * <ThemeSwitcher />
 */
"use client";

import { useEffect, useState } from "react";
import { Moon, Sun } from "lucide-react";

import { Switch } from "@/components/ui/switch";

export default function ThemeSwitcher() {
  /** Whether dark mode is currently active. @default true */
  const [isDark, setIsDark] = useState(true);

  /**
   * Initialize theme from localStorage or document class on mount.
   * Falls back to system preference if no saved preference exists.
   */
  useEffect(() => {
    // Check localStorage first (source of truth)
    const savedTheme = localStorage.getItem("theme");
    if (savedTheme) {
      setIsDark(savedTheme === "dark");
    } else {
      // Fall back to document class or system preference
      const root = document.documentElement;
      const isDarkMode = root.classList.contains("dark") ||
        (!root.classList.contains("light") && window.matchMedia("(prefers-color-scheme: dark)").matches);
      setIsDark(isDarkMode);
    }
  }, []);

  // Toggle dark class on document root
  const handleToggle = (checked: boolean) => {
    const dark = !checked; // Sun (checked) = light mode, Moon = dark mode
    setIsDark(dark);
    const root = document.documentElement;

    if (dark) {
      root.classList.add("dark");
      root.classList.remove("light");
    } else {
      root.classList.remove("dark");
      root.classList.add("light");
    }

    // Persist preference
    localStorage.setItem("theme", dark ? "dark" : "light");
  };

  return (
    <div className="flex items-center gap-2">
      <Moon className="size-4 text-muted-foreground" />
      <Switch
        checked={!isDark}
        onCheckedChange={handleToggle}
        aria-label="Toggle theme"
        className="data-[state=checked]:bg-primary"
      />
      <Sun className="size-4 text-muted-foreground" />
    </div>
  );
}
