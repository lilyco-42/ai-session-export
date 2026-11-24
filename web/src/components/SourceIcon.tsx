import type { ComponentType } from "react";
import ClaudeColor from "@lobehub/icons/es/Claude/components/Color";
import ClineMono from "@lobehub/icons/es/Cline/components/Mono";
import CursorMono from "@lobehub/icons/es/Cursor/components/Mono";
import DeepSeekColor from "@lobehub/icons/es/DeepSeek/components/Color";
import GeminiColor from "@lobehub/icons/es/Gemini/components/Color";
import GithubCopilotMono from "@lobehub/icons/es/GithubCopilot/components/Mono";
import OpenAIMono from "@lobehub/icons/es/OpenAI/components/Mono";
import WindsurfMono from "@lobehub/icons/es/Windsurf/components/Mono";
import { Bot, Code2, Orbit, Pi, Terminal } from "lucide-react";
import type { SourceId } from "@/api";
import { cn } from "@/lib/utils";

interface IconProps {
  className?: string;
  size?: number | string;
}

interface SourceIconSpec {
  Icon: ComponentType<IconProps>;
  tone?: string;
}

const SOURCE_ICONS: Record<string, SourceIconSpec> = {
  claude: { Icon: ClaudeColor },
  gemini: { Icon: GeminiColor },
  dsh: { Icon: DeepSeekColor },
  codex: { Icon: OpenAIMono, tone: "text-foreground" },
  pi: { Icon: Pi, tone: "text-amber-600 dark:text-amber-400" },
  agy: { Icon: Orbit, tone: "text-primary" },
  opencode: { Icon: Terminal, tone: "text-emerald-600 dark:text-emerald-400" },
  codearts: { Icon: Code2, tone: "text-sky-600 dark:text-sky-400" },
  cursor: { Icon: CursorMono, tone: "text-foreground" },
  copilot: { Icon: GithubCopilotMono, tone: "text-foreground" },
  "github-copilot": { Icon: GithubCopilotMono, tone: "text-foreground" },
  github_copilot: { Icon: GithubCopilotMono, tone: "text-foreground" },
  cline: { Icon: ClineMono, tone: "text-foreground" },
  windsurf: { Icon: WindsurfMono, tone: "text-teal-600 dark:text-teal-400" },
};

const FALLBACK_TONES = [
  "text-indigo-600 dark:text-indigo-400",
  "text-violet-600 dark:text-violet-400",
  "text-rose-600 dark:text-rose-400",
  "text-cyan-600 dark:text-cyan-400",
  "text-orange-600 dark:text-orange-400",
  "text-teal-600 dark:text-teal-400",
];

function fallbackTone(id: string): string {
  let hash = 0;
  for (let i = 0; i < id.length; i += 1) {
    hash = (hash * 31 + id.charCodeAt(i)) >>> 0;
  }
  return FALLBACK_TONES[hash % FALLBACK_TONES.length];
}

export interface SourceIconProps {
  source: SourceId;
  className?: string;
  size?: number;
}

export function SourceIcon({ source, className, size = 16 }: SourceIconProps) {
  const key = source.trim().toLowerCase();
  const matched = SOURCE_ICONS[key];
  if (matched) {
    const { Icon, tone } = matched;
    return <Icon size={size} className={cn("shrink-0", tone, className)} aria-hidden="true" />;
  }
  return (
    <Bot
      size={size}
      data-icon-fallback="true"
      className={cn("shrink-0", fallbackTone(key), className)}
      aria-hidden="true"
    />
  );
}
