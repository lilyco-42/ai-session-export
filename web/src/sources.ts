import type { BuiltInSourceId, SessionSummary, SourceId, SourceStatus, SourceSummary } from "@/api";

export const BUILTIN_SOURCE_IDS: readonly BuiltInSourceId[] = [
  "codearts",
  "opencode",
  "agy",
  "codex",
  "dsh",
  "claude",
  "gemini",
  "pi",
];

export const BUILTIN_SOURCE_LABELS: Record<BuiltInSourceId, string> = {
  codearts: "CodeArts / CodeBuddy",
  opencode: "OpenCode",
  agy: "AGY / Antigravity",
  codex: "Codex CLI",
  dsh: "DeepSeek Harness",
  claude: "Claude Code",
  gemini: "Gemini CLI",
  pi: "Pi",
};

const BUILTIN_SET: ReadonlySet<string> = new Set(BUILTIN_SOURCE_IDS);

export interface SourceOption {
  id: SourceId;
  label: string;
  status: SourceStatus;
  sessionCount: number;
}

export interface GroupedSourceOptions {
  builtIn: SourceOption[];
  custom: SourceOption[];
}

export function isBuiltInSourceId(id: string): id is BuiltInSourceId {
  return BUILTIN_SET.has(id);
}

export function buildSourceLabelMap(sources: readonly SourceSummary[]): Map<SourceId, string> {
  const map = new Map<SourceId, string>();
  for (const id of BUILTIN_SOURCE_IDS) {
    map.set(id, BUILTIN_SOURCE_LABELS[id]);
  }
  for (const source of sources) {
    const trimmed = source.label.trim();
    if (trimmed && (trimmed !== source.id || !isBuiltInSourceId(source.id))) {
      map.set(source.id, trimmed);
    } else if (!map.has(source.id)) {
      map.set(source.id, source.id);
    }
  }
  return map;
}

export function resolveSourceLabel(id: SourceId, labelMap: ReadonlyMap<SourceId, string>): string {
  const fromMap = labelMap.get(id);
  if (fromMap) return fromMap;
  return isBuiltInSourceId(id) ? BUILTIN_SOURCE_LABELS[id] : id;
}

export function formatSourceStatus(status: SourceStatus, count: number): string {
  if (status === "ready") return `就绪 · ${count} 个会话`;
  if (status === "missing") return "未发现本地存储";
  return "读取失败";
}

export function collectSourceOptions(
  sources: readonly SourceSummary[],
  sessions: readonly SessionSummary[],
  labelMap: ReadonlyMap<SourceId, string>,
  activeFilter: SourceId | "all",
): GroupedSourceOptions {
  const byId = new Map<SourceId, SourceOption>();
  for (const source of sources) {
    byId.set(source.id, {
      id: source.id,
      label: resolveSourceLabel(source.id, labelMap),
      status: source.status,
      sessionCount: source.sessionCount,
    });
  }
  for (const session of sessions) {
    if (!byId.has(session.source)) {
      byId.set(session.source, {
        id: session.source,
        label: resolveSourceLabel(session.source, labelMap),
        status: "ready",
        sessionCount: 0,
      });
    }
  }
  if (activeFilter !== "all" && !byId.has(activeFilter)) {
    byId.set(activeFilter, {
      id: activeFilter,
      label: resolveSourceLabel(activeFilter, labelMap),
      status: "missing",
      sessionCount: 0,
    });
  }
  const builtIn: SourceOption[] = [];
  const custom: SourceOption[] = [];
  for (const option of byId.values()) {
    if (isBuiltInSourceId(option.id)) builtIn.push(option);
    else custom.push(option);
  }
  return { builtIn, custom };
}
