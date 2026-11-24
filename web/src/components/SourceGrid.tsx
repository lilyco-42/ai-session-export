import { useMemo, useState } from "react";
import { RefreshCw, Search, X } from "lucide-react";
import type { SourceId, SourceSummary } from "@/api";
import { SourceIcon } from "@/components/SourceIcon";
import { Button } from "@/components/ui/button";
import { formatSourceStatus, isBuiltInSourceId, resolveSourceLabel } from "@/sources";

type Scope = "all" | "ready" | "custom" | "unavailable";

interface SourceGridProps {
  sources: readonly SourceSummary[];
  labelMap: ReadonlyMap<SourceId, string>;
  activeFilter: SourceId | "all";
  busy: boolean;
  onSelectSource: (id: SourceId) => void;
  onRefresh: () => void;
}

function statusDotClass(status: SourceSummary["status"]): string {
  if (status === "ready") return "bg-emerald-500";
  if (status === "error") return "bg-destructive";
  return "bg-muted-foreground/40";
}

function statusTextClass(status: SourceSummary["status"]): string {
  if (status === "ready") return "text-emerald-600 dark:text-emerald-400";
  if (status === "error") return "text-destructive";
  return "text-muted-foreground";
}

export function SourceGrid({
  sources,
  labelMap,
  activeFilter,
  busy,
  onSelectSource,
  onRefresh,
}: SourceGridProps) {
  const [query, setQuery] = useState("");
  const [scope, setScope] = useState<Scope>("all");

  const stats = useMemo(() => {
    let ready = 0;
    let custom = 0;
    let unavailable = 0;
    let totalSessions = 0;
    for (const s of sources) {
      if (s.status === "ready") ready += 1;
      else unavailable += 1;
      if (!isBuiltInSourceId(s.id)) custom += 1;
      totalSessions += s.sessionCount;
    }
    return { ready, custom, unavailable, totalSessions };
  }, [sources]);

  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return sources.filter((s) => {
      if (scope === "ready" && s.status !== "ready") return false;
      if (scope === "custom" && isBuiltInSourceId(s.id)) return false;
      if (scope === "unavailable" && s.status === "ready") return false;
      if (!needle) return true;
      const label = resolveSourceLabel(s.id, labelMap).toLowerCase();
      return label.includes(needle) || s.id.toLowerCase().includes(needle);
    });
  }, [sources, labelMap, query, scope]);

  const scopes: { id: Scope; text: string; count: number; show: boolean }[] = [
    { id: "all", text: "全部", count: sources.length, show: true },
    { id: "ready", text: "就绪", count: stats.ready, show: true },
    { id: "custom", text: "自定义", count: stats.custom, show: stats.custom > 0 },
    { id: "unavailable", text: "未就绪", count: stats.unavailable, show: stats.unavailable > 0 },
  ];

  return (
    <section className="space-y-3">
      <div className="flex flex-wrap items-center gap-2 rounded-xl border border-border bg-card p-3 shadow-xs">
        <div className="relative min-w-0 flex-1 basis-48">
          <Search className="pointer-events-none absolute top-1/2 left-2.5 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
          <input
            type="search"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            aria-label="筛选数据源"
            placeholder={`筛选 ${sources.length} 个数据源（名称或 ID）...`}
            className="h-8 w-full rounded-md border border-input bg-background pr-7 pl-8 text-xs outline-none focus-visible:ring-2 focus-visible:ring-ring"
          />
          {query && (
            <button
              type="button"
              onClick={() => setQuery("")}
              aria-label="清空搜索"
              className="absolute top-1/2 right-2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
            >
              <X className="h-3.5 w-3.5" />
            </button>
          )}
        </div>

        <div className="flex max-w-full items-center gap-1 overflow-x-auto">
          {scopes
            .filter((item) => item.show)
            .map((item) => (
              <button
                key={item.id}
                type="button"
                onClick={() => setScope(item.id)}
                className={`inline-flex shrink-0 cursor-pointer items-center gap-1 rounded-md px-2.5 py-1 text-xs font-medium transition-colors ${
                  scope === item.id
                    ? "bg-primary/10 text-primary"
                    : "text-muted-foreground hover:bg-accent hover:text-foreground"
                }`}
              >
                <span>{item.text}</span>
                <span className="font-mono text-[11px] opacity-80">{item.count}</span>
              </button>
            ))}
        </div>

        <div className="ml-auto flex shrink-0 items-center gap-2">
          <span className="hidden text-xs text-muted-foreground sm:inline">
            共 {stats.totalSessions} 个会话
          </span>
          <Button size="sm" variant="secondary" onClick={onRefresh} disabled={busy}>
            <RefreshCw className={`h-3.5 w-3.5 ${busy ? "animate-spin" : ""}`} /> 刷新
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
        {filtered.map((s) => {
          const label = resolveSourceLabel(s.id, labelMap);
          const custom = !isBuiltInSourceId(s.id);
          const active = activeFilter === s.id;
          return (
            <button
              key={s.id}
              type="button"
              title={`${label} (${s.id})`}
              onClick={() => onSelectSource(s.id)}
              className={`min-w-0 cursor-pointer rounded-xl border bg-card p-4 text-left shadow-xs transition-colors hover:border-primary/50 ${
                active ? "border-primary ring-1 ring-primary/30" : "border-border"
              }`}
            >
              <div className="flex items-start gap-2.5">
                <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-muted/70">
                  <SourceIcon source={s.id} size={18} />
                </div>
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-1.5">
                    <span className="truncate text-sm font-semibold">{label}</span>
                    {custom && (
                      <span className="shrink-0 rounded-full bg-secondary px-1.5 py-0.5 text-[10px] font-medium text-secondary-foreground">
                        自定义
                      </span>
                    )}
                  </div>
                  <div className="truncate font-mono text-[11px] text-muted-foreground">{s.id}</div>
                </div>
              </div>
              <div className={`mt-3 flex items-center gap-1.5 text-xs ${statusTextClass(s.status)}`}>
                <span className={`h-1.5 w-1.5 shrink-0 rounded-full ${statusDotClass(s.status)}`} />
                <span className="truncate">{formatSourceStatus(s.status, s.sessionCount)}</span>
              </div>
            </button>
          );
        })}
      </div>

      {!filtered.length && (
        <div className="rounded-xl border border-dashed border-border bg-card px-4 py-10 text-center text-sm text-muted-foreground">
          {sources.length ? "没有匹配当前筛选条件的数据源。" : "暂无可用数据源；请检查本地导出服务。"}
        </div>
      )}
    </section>
  );
}
