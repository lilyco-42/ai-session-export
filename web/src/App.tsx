import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Command, Database, Download, Moon, ShieldCheck, Sun } from "lucide-react";
import { Toaster, toast } from "sonner";
import {
  ApiError,
  exportSessions,
  loadSessions,
  loadSources,
  type ExportFormat,
  type SessionSummary,
  type SourceId,
  type SourceSummary,
} from "@/api";
import { CommandMenu } from "@/CommandMenu";
import { ExportPanel } from "@/components/ExportPanel";
import { SourceGrid } from "@/components/SourceGrid";
import { Button } from "@/components/ui/button";
import { buildSourceLabelMap } from "@/sources";

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
  const [filter, setFilter] = useState<SourceId | "all">("all");
  const [selected, setSelected] = useState<string[]>([]);
  const [format, setFormat] = useState<ExportFormat>("markdown");
  const [busy, setBusy] = useState(false);
  const requestSeq = useRef(0);

  useEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
  }, [dark]);

  const labelMap = useMemo(() => buildSourceLabelMap(sources), [sources]);

  const refresh = useCallback(async (activeFilter: SourceId | "all") => {
    const seq = ++requestSeq.current;
    setBusy(true);
    try {
      const [sRows, rows] = await Promise.all([loadSources(), loadSessions(activeFilter)]);
      if (seq !== requestSeq.current) return;
      setSources(sRows);
      setSessions(rows);
      setSelected([]);
    } catch (err) {
      if (seq === requestSeq.current) toast.error(describeError(err));
    } finally {
      if (seq === requestSeq.current) setBusy(false);
    }
  }, []);

  useEffect(() => {
    void refresh(filter);
  }, [filter, refresh]);

  const handleSelectSource = useCallback((id: SourceId | "all") => {
    setFilter(id);
    setTab("export");
  }, []);

  async function runExport(ids: string[]) {
    if (!ids.length) {
      toast.warning("请先选择会话");
      return;
    }
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
      <CommandMenu
        open={cmdOpen}
        onOpenChange={setCmdOpen}
        sources={sources}
        labelMap={labelMap}
        onSelectTab={setTab}
        onSelectSource={handleSelectSource}
        onToggleTheme={() => setDark((v) => !v)}
        dark={dark}
      />

      <aside className="hidden w-60 shrink-0 flex-col border-r border-sidebar-border bg-sidebar md:flex">
        <div className="flex h-15 items-center gap-2.5 border-b border-sidebar-border px-4">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary text-sm font-bold text-primary-foreground shadow-xs">
            AI
          </div>
          <div className="leading-tight">
            <div className="text-sm font-bold">会话导出器</div>
            <div className="text-[11px] text-muted-foreground">Lilyco · 本地只读</div>
          </div>
        </div>
        <nav className="flex-1 space-y-1 p-2.5">
          <button
            type="button"
            onClick={() => setTab("sources")}
            className={`flex w-full cursor-pointer items-center justify-between gap-2 rounded-lg px-3 py-2 text-sm font-medium ${
              tab === "sources" ? "bg-primary/10 text-primary" : "text-muted-foreground hover:bg-accent"
            }`}
          >
            <span className="flex items-center gap-2.5">
              <Database className="h-4 w-4" /> 数据源
            </span>
            {sources.length > 0 && <span className="font-mono text-xs opacity-75">{sources.length}</span>}
          </button>
          <button
            type="button"
            onClick={() => setTab("export")}
            className={`flex w-full cursor-pointer items-center justify-between gap-2 rounded-lg px-3 py-2 text-sm font-medium ${
              tab === "export" ? "bg-primary/10 text-primary" : "text-muted-foreground hover:bg-accent"
            }`}
          >
            <span className="flex items-center gap-2.5">
              <Download className="h-4 w-4" /> 导出队列
            </span>
            {sessions.length > 0 && <span className="font-mono text-xs opacity-75">{sessions.length}</span>}
          </button>
        </nav>
        <div className="border-t border-sidebar-border p-3 text-[11px] text-muted-foreground">
          <div className="flex items-center gap-1.5">
            <ShieldCheck className="h-3.5 w-3.5 shrink-0" /> 原始会话库只读打开
          </div>
          <div className="mt-1">支持内置工具与自定义 JSONL 来源</div>
        </div>
      </aside>

      <div className="flex min-w-0 flex-1 flex-col">
        <header className="sticky top-0 z-10 flex h-15 items-center justify-between gap-2 border-b border-border bg-card/85 px-4 backdrop-blur-sm md:px-6">
          <div className="flex min-w-0 items-center gap-2">
            <h1 className="truncate text-base font-bold">{tab === "sources" ? "本地数据源" : "导出队列"}</h1>
            <div className="flex shrink-0 items-center rounded-lg border border-border bg-muted/50 p-0.5 md:hidden">
              <button
                type="button"
                onClick={() => setTab("sources")}
                className={`cursor-pointer rounded-md px-2 py-1 text-xs font-medium ${
                  tab === "sources" ? "bg-background text-foreground shadow-xs" : "text-muted-foreground"
                }`}
              >
                数据源
              </button>
              <button
                type="button"
                onClick={() => setTab("export")}
                className={`cursor-pointer rounded-md px-2 py-1 text-xs font-medium ${
                  tab === "export" ? "bg-background text-foreground shadow-xs" : "text-muted-foreground"
                }`}
              >
                导出
              </button>
            </div>
          </div>

          <div className="flex shrink-0 items-center gap-2">
            <Button variant="secondary" size="sm" onClick={() => setCmdOpen(true)}>
              <Command className="h-3.5 w-3.5" /> <span className="hidden sm:inline">快捷命令 ⌘K</span>
            </Button>
            <Button
              variant="secondary"
              size="icon"
              onClick={() => setDark((v) => !v)}
              title="切换明暗主题"
              aria-label="切换明暗主题"
            >
              {dark ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
            </Button>
          </div>
        </header>

        <main className="mx-auto w-full max-w-6xl min-w-0 space-y-4 p-4 md:p-6">
          {tab === "sources" ? (
            <SourceGrid
              sources={sources}
              labelMap={labelMap}
              activeFilter={filter}
              busy={busy}
              onSelectSource={handleSelectSource}
              onRefresh={() => void refresh(filter)}
            />
          ) : (
            <ExportPanel
              sources={sources}
              sessions={sessions}
              labelMap={labelMap}
              filter={filter}
              format={format}
              selected={selected}
              busy={busy}
              onFilterChange={setFilter}
              onFormatChange={setFormat}
              onSelectedChange={setSelected}
              onRefresh={() => void refresh(filter)}
              onExport={(ids) => void runExport(ids)}
            />
          )}
        </main>
      </div>
    </div>
  );
}

export default App;
