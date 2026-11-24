import { useEffect, useMemo, useState } from "react";
import { flexRender, getCoreRowModel, useReactTable, type ColumnDef } from "@tanstack/react-table";
import { RefreshCw, Search, X } from "lucide-react";
import { searchSessions, type ExportFormat, type SessionSummary, type SourceId, type SourceSummary } from "@/api";
import { SessionPreview } from "@/components/SessionPreview";
import { SourceIcon } from "@/components/SourceIcon";
import { Button } from "@/components/ui/button";
import { collectSourceOptions, resolveSourceLabel, type SourceOption } from "@/sources";

interface ExportPanelProps {
  sources: readonly SourceSummary[];
  sessions: readonly SessionSummary[];
  labelMap: ReadonlyMap<SourceId, string>;
  filter: SourceId | "all";
  format: ExportFormat;
  selected: readonly string[];
  busy: boolean;
  onFilterChange: (next: SourceId | "all") => void;
  onFormatChange: (next: ExportFormat) => void;
  onSelectedChange: (next: string[]) => void;
  onRefresh: () => void;
  onExport: (ids: string[]) => void;
}

export function ExportPanel({
  sources,
  sessions,
  labelMap,
  filter,
  format,
  selected,
  busy,
  onFilterChange,
  onFormatChange,
  onSelectedChange,
  onRefresh,
  onExport,
}: ExportPanelProps) {
  const [query, setQuery] = useState("");
  const [showAllPills, setShowAllPills] = useState(false);
  const [remote, setRemote] = useState<{query: string; filter: SourceId; rows: SessionSummary[]} | null>(null);
  const [searchError, setSearchError] = useState("");
  const [preview, setPreview] = useState<string | null>(null);
  useEffect(() => {
    const text = query.trim();
    let active = true;
    setRemote(null);
    setSearchError("");
    if (!text) return;
    const timer = setTimeout(() => {
      searchSessions(text, filter).then(rows => {
        if (active) setRemote({ query: text, filter, rows });
      }).catch(() => {
        if (active) setSearchError("全文搜索失败；当前仅显示标题及来源匹配。");
      });
    }, 250);
    return () => { active = false; clearTimeout(timer); };
  }, [query, filter, sessions]);

  const grouped = useMemo(
    () => collectSourceOptions(sources, sessions, labelMap, filter),
    [sources, sessions, labelMap, filter],
  );

  const allOptions = useMemo<SourceOption[]>(
    () => [...grouped.builtIn, ...grouped.custom],
    [grouped],
  );

  const totalSessions = useMemo(
    () => sources.reduce((sum, item) => sum + item.sessionCount, 0),
    [sources],
  );

  const { visiblePills, hiddenCount } = useMemo(() => {
    if (showAllPills || allOptions.length <= 8) {
      return { visiblePills: allOptions, hiddenCount: 0 };
    }
    const primary = allOptions.filter((opt) => opt.status === "ready" || opt.id === filter);
    return {
      visiblePills: primary,
      hiddenCount: Math.max(0, allOptions.length - primary.length),
    };
  }, [allOptions, filter, showAllPills]);

  const visibleSessions = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) return [...sessions];
    if (remote?.query === query.trim() && remote.filter === filter) return remote.rows;
    return sessions.filter((row) => {
      const label = resolveSourceLabel(row.source, labelMap).toLowerCase();
      return (
        row.title.toLowerCase().includes(needle) ||
        row.source.toLowerCase().includes(needle) ||
        label.includes(needle)
      );
    });
  }, [sessions, labelMap, query, remote, filter]);

  const selectedSet = useMemo(() => new Set(selected), [selected]);
  const visibleSelectedCount = useMemo(
    () => visibleSessions.filter((row) => selectedSet.has(row.id)).length,
    [visibleSessions, selectedSet],
  );

  const columns = useMemo<ColumnDef<SessionSummary>[]>(
    () => [
      {
        id: "select",
        header: () => (
          <input
            type="checkbox"
            aria-label="全选当前会话"
            checked={visibleSessions.length > 0 && visibleSelectedCount === visibleSessions.length}
            onChange={(e) => {
              if (e.target.checked) {
                const merged = new Set(selected);
                for (const row of visibleSessions) merged.add(row.id);
                onSelectedChange([...merged]);
              } else {
                const visibleIds = new Set(visibleSessions.map((row) => row.id));
                onSelectedChange(selected.filter((id) => !visibleIds.has(id)));
              }
            }}
          />
        ),
        cell: ({ row }) => (
          <input
            type="checkbox"
            aria-label={`选择 ${row.original.title}`}
            checked={selectedSet.has(row.original.id)}
            onChange={(e) => {
              const id = row.original.id;
              onSelectedChange(e.target.checked ? [...selected, id] : selected.filter((v) => v !== id));
            }}
          />
        ),
      },
      {
        accessorKey: "source",
        header: "工具",
        cell: ({ getValue }) => {
          const id = String(getValue());
          const label = resolveSourceLabel(id, labelMap);
          return (
            <span
              title={`${label} (${id})`}
              className="inline-flex max-w-[180px] items-center gap-1.5 rounded-full bg-muted px-2.5 py-0.5 text-xs font-medium"
            >
              <SourceIcon source={id} size={14} />
              <span className="truncate">{label}</span>
            </span>
          );
        },
      },
      {
        accessorKey: "title",
        header: "会话标题",
        cell: ({ getValue, row }) => <button type="button" className="max-w-md truncate text-left font-medium hover:underline" onClick={() => setPreview(row.original.id)}>{String(getValue())}</button>,
      },
      {
        accessorKey: "updatedAt",
        header: "更新时间",
        cell: ({ getValue }) => <span className="whitespace-nowrap font-mono text-xs">{String(getValue())}</span>,
      },
    ],
    [visibleSessions, visibleSelectedCount, selected, selectedSet, labelMap, onSelectedChange],
  );

  const table = useReactTable({ data: visibleSessions, columns, getCoreRowModel: getCoreRowModel() });

  return (
    <section className="min-w-0 rounded-xl border border-border bg-card shadow-xs">
      {allOptions.length > 0 && (
        <div className="border-b border-border px-3 py-2">
          <div className="flex items-center gap-1.5 overflow-x-auto pb-0.5">
            <button
              type="button"
              onClick={() => onFilterChange("all")}
              className={`inline-flex shrink-0 cursor-pointer items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs font-medium transition-colors ${
                filter === "all"
                  ? "border-primary/40 bg-primary/10 text-primary"
                  : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground"
              }`}
            >
              <span>全部工具</span>
              <span className="font-mono text-[11px] opacity-80">{totalSessions}</span>
            </button>

            {visiblePills.map((opt) => (
              <button
                key={opt.id}
                type="button"
                title={`${opt.label} (${opt.id})`}
                onClick={() => onFilterChange(opt.id)}
                className={`inline-flex shrink-0 cursor-pointer items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs font-medium transition-colors ${
                  filter === opt.id
                    ? "border-primary/40 bg-primary/10 text-primary"
                    : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground"
                }`}
              >
                <SourceIcon source={opt.id} size={13} />
                <span className="max-w-[140px] truncate">{opt.label}</span>
                <span className="font-mono text-[11px] opacity-75">{opt.sessionCount}</span>
              </button>
            ))}

            {hiddenCount > 0 && (
              <button
                type="button"
                onClick={() => setShowAllPills(true)}
                className="inline-flex shrink-0 cursor-pointer items-center rounded-full border border-dashed border-border px-2.5 py-1 text-xs text-muted-foreground hover:bg-accent hover:text-foreground"
              >
                +{hiddenCount} 更多
              </button>
            )}
            {showAllPills && allOptions.length > 8 && (
              <button
                type="button"
                onClick={() => setShowAllPills(false)}
                className="inline-flex shrink-0 cursor-pointer items-center rounded-full border border-dashed border-border px-2.5 py-1 text-xs text-muted-foreground hover:bg-accent hover:text-foreground"
              >
                收起未就绪
              </button>
            )}
          </div>
        </div>
      )}

      <div className="flex flex-wrap items-center gap-2 border-b border-border p-3">
        <select
          aria-label="筛选工具"
          value={filter}
          onChange={(e) => onFilterChange(e.target.value)}
          className="h-8 max-w-full rounded-md border border-input bg-background px-2 text-xs"
        >
          <option value="all">全部工具</option>
          {grouped.builtIn.length > 0 && (
            <optgroup label="内置来源">
              {grouped.builtIn.map((opt) => (
                <option key={opt.id} value={opt.id}>
                  {opt.label} ({opt.sessionCount})
                </option>
              ))}
            </optgroup>
          )}
          {grouped.custom.length > 0 && (
            <optgroup label="自定义来源">
              {grouped.custom.map((opt) => (
                <option key={opt.id} value={opt.id}>
                  {opt.label} ({opt.sessionCount})
                </option>
              ))}
            </optgroup>
          )}
        </select>

        <select
          aria-label="导出格式"
          value={format}
          onChange={(e) => onFormatChange(e.target.value as ExportFormat)}
          className="h-8 rounded-md border border-input bg-background px-2 text-xs"
        >
          <option value="markdown">Markdown</option>
          <option value="jsonl">JSONL</option>
          <option value="raw">原始归档</option>
          <option value="html">HTML</option>
        </select>

        <div className="relative min-w-0 flex-1 basis-40">
          <Search className="pointer-events-none absolute top-1/2 left-2.5 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
          <input
            type="search"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            aria-label="搜索会话"
            placeholder="搜索标题或对话内容..."
            className="h-8 w-full rounded-md border border-input bg-background pr-7 pl-8 text-xs outline-none focus-visible:ring-2 focus-visible:ring-ring"
          />
          {query && (
            <button
              type="button"
              onClick={() => setQuery("")}
              aria-label="清空会话搜索"
              className="absolute top-1/2 right-2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
            >
              <X className="h-3.5 w-3.5" />
            </button>
          )}
        </div>

        <Button size="sm" variant="secondary" onClick={onRefresh} disabled={busy}>
          <RefreshCw className={`h-3.5 w-3.5 ${busy ? "animate-spin" : ""}`} /> 刷新
        </Button>

        <div className="ml-auto flex shrink-0 items-center gap-2">
          <span className="hidden text-xs text-muted-foreground sm:inline">
            已选 {selected.length} / {visibleSessions.length}
          </span>
          <Button size="sm" variant="outline" onClick={() => onExport([...selected])} disabled={busy}>
            导出所选
          </Button>
          <Button
            size="sm"
            onClick={() => onExport(visibleSessions.map((row) => row.id))}
            disabled={busy}
          >
            导出全部
          </Button>
        </div>
      </div>

      {searchError && <p role="alert" className="px-3 py-2 text-xs text-muted-foreground">{searchError}</p>}
      {remote && remote.rows.length === 200 && <p className="px-3 py-2 text-xs text-muted-foreground">最多显示 200 条搜索结果，请缩小搜索范围。</p>}
      <div className="w-full overflow-x-auto">
        <table className="w-full min-w-[640px] border-collapse text-left text-sm">
          <thead className="bg-muted/60 text-xs text-muted-foreground">
            {table.getHeaderGroups().map((hg) => (
              <tr key={hg.id}>
                {hg.headers.map((h) => (
                  <th key={h.id} className="border-b border-border px-4 py-2.5 font-semibold">
                    {flexRender(h.column.columnDef.header, h.getContext())}
                  </th>
                ))}
              </tr>
            ))}
          </thead>
          <tbody>
            {table.getRowModel().rows.map((r) => (
              <tr key={r.id} className="border-b border-border last:border-none hover:bg-muted/40">
                {r.getVisibleCells().map((c) => (
                  <td key={c.id} className="px-4 py-3">
                    {flexRender(c.column.columnDef.cell, c.getContext())}
                  </td>
                ))}
              </tr>
            ))}
            {!visibleSessions.length && (
              <tr>
                <td colSpan={4} className="px-4 py-10 text-center text-sm text-muted-foreground">
                  {sessions.length ? "没有匹配当前搜索条件的会话。" : "暂无会话；请先确认本地存储路径。"}
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
      {preview && <SessionPreview key={preview} id={preview} onClose={() => setPreview(null)} />}
    </section>
  );
}
