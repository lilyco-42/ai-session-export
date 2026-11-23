import { useEffect, useMemo, useState } from "react";
import { flexRender, getCoreRowModel, useReactTable, type ColumnDef } from "@tanstack/react-table";
import { Command, Database, Download, FolderOpen, Moon, RefreshCw, ShieldCheck, Sun } from "lucide-react";
import { Toaster, toast } from "sonner";
import { Button } from "@/components/ui/button";
import { CommandMenu } from "@/CommandMenu";
import { ApiError, exportSessions, loadSessions, loadSources, type ExportFormat, type SessionSummary, type SourceSummary, type ToolId } from "@/api";

const SOURCE_LABELS: Record<ToolId, string> = {
  codearts: "CodeArts / CodeBuddy",
  opencode: "OpenCode",
  agy: "AGY / Antigravity",
  codex: "Codex CLI",
  dsh: "DeepSeek Harness",
};

function isToolId(v: string): v is ToolId {
  return v === "codearts" || v === "opencode" || v === "agy" || v === "codex" || v === "dsh";
}

function describeError(err: unknown): string {
  if (err instanceof ApiError) return err.shape.reason_zh + "：" + err.shape.repair_zh;
  return "操作失败，请稍后重试。";
}

export function App() {
  const [tab, setTab] = useState<"sources" | "export">("sources");
  const [dark, setDark] = useState(false);
  const [cmdOpen, setCmdOpen] = useState(false);
  const [sources, setSources] = useState<SourceSummary[]>([]);
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [filter, setFilter] = useState<ToolId | "all">("all");
  const [selected, setSelected] = useState<string[]>([]);
  const [format, setFormat] = useState<ExportFormat>("markdown");
  const [busy, setBusy] = useState(false);

  useEffect(() => { document.documentElement.classList.toggle("dark", dark); }, [dark]);

  async function refresh() {
    setBusy(true);
    try {
      const [sRows, rows] = await Promise.all([loadSources(), loadSessions(filter)]);
      setSources(sRows);
      setSessions(rows);
      setSelected([]);
    } catch (err) {
      toast.error(describeError(err));
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => { void refresh(); }, [filter]);

  const columns = useMemo<ColumnDef<SessionSummary>[]>(() => [
    {
      id: "select",
      header: () => <input type="checkbox" aria-label="全选当前会话" checked={sessions.length > 0 && selected.length === sessions.length} onChange={(e) => setSelected(e.target.checked ? sessions.map((s) => s.id) : [])} />,
      cell: ({ row }) => <input type="checkbox" aria-label={`选择 ${row.original.title}`} checked={selected.includes(row.original.id)} onChange={(e) => setSelected(e.target.checked ? [...selected, row.original.id] : selected.filter((v) => v !== row.original.id))} />,
    },
    {
      accessorKey: "source",
      header: "工具",
      cell: ({ getValue }) => {
        const value = String(getValue());
        return <span className="rounded-full bg-muted px-2 py-0.5 text-xs">{isToolId(value) ? SOURCE_LABELS[value] : value}</span>;
      },
    },
    { accessorKey: "title", header: "会话标题" },
    { accessorKey: "updatedAt", header: "更新时间", cell: ({ getValue }) => <span className="font-mono text-xs">{String(getValue())}</span> },
  ], [sessions, selected]);

  const table = useReactTable({ data: sessions, columns, getCoreRowModel: getCoreRowModel() });

  async function runExport(all: boolean) {
    const ids = all ? sessions.map((s) => s.id) : selected;
    if (!ids.length) { toast.warning("请先选择会话"); return; }
    setBusy(true);
    try {
      const result = await exportSessions(ids, format);
      window.location.href = result.downloadUrl;
      toast.success(`已生成 ${ids.length} 个会话的 ${format} 导出文件`);
    } catch (err) {
      toast.error(describeError(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex min-h-screen bg-background text-foreground">
      <Toaster richColors position="bottom-right" />
      <CommandMenu open={cmdOpen} onOpenChange={setCmdOpen} onSelectTab={setTab} onToggleTheme={() => setDark((v) => !v)} dark={dark} />

      <aside className="hidden w-60 shrink-0 flex-col border-r border-sidebar-border bg-sidebar md:flex">
        <div className="flex h-15 items-center gap-2.5 border-b border-sidebar-border px-4">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary text-sm font-bold text-primary-foreground shadow-xs">AI</div>
          <div className="leading-tight">
            <div className="text-sm font-bold">会话导出器</div>
            <div className="text-[11px] text-muted-foreground">Lilyco · 本地只读</div>
          </div>
        </div>
        <nav className="flex-1 space-y-1 p-2.5">
          <button onClick={() => setTab("sources")} className={`flex w-full cursor-pointer items-center gap-2.5 rounded-lg px-3 py-2 text-sm font-medium ${tab === "sources" ? "bg-primary/10 text-primary" : "text-muted-foreground hover:bg-accent"}`}>
            <Database className="h-4 w-4" /> 数据源
          </button>
          <button onClick={() => setTab("export")} className={`flex w-full cursor-pointer items-center gap-2.5 rounded-lg px-3 py-2 text-sm font-medium ${tab === "export" ? "bg-primary/10 text-primary" : "text-muted-foreground hover:bg-accent"}`}>
            <Download className="h-4 w-4" /> 导出队列
          </button>
        </nav>
        <div className="border-t border-sidebar-border p-3 text-[11px] text-muted-foreground">
          <div className="flex items-center gap-1.5"><ShieldCheck className="h-3.5 w-3.5" /> 原始会话库只读打开</div>
          <div className="mt-1">不修改 Codex / AGY / DSH 存储</div>
        </div>
      </aside>

      <div className="flex min-w-0 flex-1 flex-col">
        <header className="sticky top-0 z-10 flex h-15 items-center justify-between gap-2 border-b border-border bg-card/85 px-4 backdrop-blur-sm md:px-6">
          <h1 className="truncate text-base font-bold">{tab === "sources" ? "本地数据源" : "导出队列"}</h1>
          <div className="flex shrink-0 items-center gap-2">
            <Button variant="secondary" size="sm" onClick={() => setCmdOpen(true)}><Command className="h-3.5 w-3.5" /> <span className="hidden sm:inline">快捷命令 ⌘K</span></Button>
            <Button variant="secondary" size="icon" onClick={() => setDark((v) => !v)} title="切换明暗主题">{dark ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}</Button>
          </div>
        </header>

        <main className="mx-auto w-full max-w-6xl space-y-4 p-4 md:p-6">
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-5">
            {sources.map((s) => (
              <button key={s.id} onClick={() => { setFilter(s.id); setTab("export"); }} className="min-w-0 rounded-xl border border-border bg-card p-4 text-left shadow-xs transition-colors hover:border-primary/50">
                <div className="flex items-center gap-2">
                  <FolderOpen className="h-4 w-4 shrink-0 text-primary" />
                  <span className="truncate text-sm font-semibold">{s.label}</span>
                </div>
                <div className={`mt-2 text-xs ${s.status === "ready" ? "text-emerald-600 dark:text-emerald-400" : "text-muted-foreground"}`}>
                  {s.status === "ready" ? `就绪 · ${s.sessionCount} 个会话` : s.status === "missing" ? "未发现本地存储" : "读取失败"}
                </div>
              </button>
            ))}
          </div>

          {tab === "export" && (
            <section className="rounded-xl border border-border bg-card shadow-xs">
              <div className="flex flex-wrap items-center gap-2 border-b border-border p-3">
                <select aria-label="筛选工具" value={filter} onChange={(e) => setFilter(e.target.value as ToolId | "all")} className="h-8 rounded-md border border-input bg-background px-2 text-xs">
                  <option value="all">全部工具</option>
                  {Object.entries(SOURCE_LABELS).map(([k, v]) => <option key={k} value={k}>{v}</option>)}
                </select>
                <select aria-label="导出格式" value={format} onChange={(e) => setFormat(e.target.value as ExportFormat)} className="h-8 rounded-md border border-input bg-background px-2 text-xs">
                  <option value="markdown">Markdown</option><option value="jsonl">JSONL</option><option value="raw">原始归档</option><option value="html">HTML</option>
                </select>
                <Button size="sm" variant="secondary" onClick={() => void refresh()} disabled={busy}><RefreshCw className={`h-3.5 w-3.5 ${busy ? "animate-spin" : ""}`} /> 刷新</Button>
                <div className="ml-auto flex shrink-0 items-center gap-2">
                  <span className="hidden text-xs text-muted-foreground sm:inline">已选 {selected.length} / {sessions.length}</span>
                  <Button size="sm" variant="outline" onClick={() => void runExport(false)} disabled={busy}>导出所选</Button>
                  <Button size="sm" onClick={() => void runExport(true)} disabled={busy}>导出全部</Button>
                </div>
              </div>
              <div className="w-full overflow-x-auto">
                <table className="w-full min-w-[640px] border-collapse text-left text-sm">
                  <thead className="bg-muted/60 text-xs text-muted-foreground">
                    {table.getHeaderGroups().map((hg) => <tr key={hg.id}>{hg.headers.map((h) => <th key={h.id} className="border-b border-border px-4 py-2.5 font-semibold">{flexRender(h.column.columnDef.header, h.getContext())}</th>)}</tr>)}
                  </thead>
                  <tbody>
                    {table.getRowModel().rows.map((r) => <tr key={r.id} className="border-b border-border last:border-none hover:bg-muted/40">{r.getVisibleCells().map((c) => <td key={c.id} className="px-4 py-3">{flexRender(c.column.columnDef.cell, c.getContext())}</td>)}</tr>)}
                    {!sessions.length && <tr><td colSpan={4} className="px-4 py-10 text-center text-sm text-muted-foreground">暂无会话；请先确认本地存储路径。</td></tr>}
                  </tbody>
                </table>
              </div>
            </section>
          )}
        </main>
      </div>
    </div>
  );
}
export default App;
