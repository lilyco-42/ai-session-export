import { useEffect } from "react";
import { Command } from "cmdk";
import { Database, Download, Layers, Moon, Search, Sun } from "lucide-react";
import type { SourceId, SourceSummary } from "@/api";
import { SourceIcon } from "@/components/SourceIcon";
import { resolveSourceLabel } from "@/sources";

interface Props {
  open: boolean;
  onOpenChange: (v: boolean) => void;
  sources: readonly SourceSummary[];
  labelMap: ReadonlyMap<SourceId, string>;
  onSelectTab: (id: "sources" | "export") => void;
  onSelectSource: (id: SourceId | "all") => void;
  onToggleTheme: () => void;
  dark: boolean;
}

export function CommandMenu({
  open,
  onOpenChange,
  sources,
  labelMap,
  onSelectTab,
  onSelectSource,
  onToggleTheme,
  dark,
}: Props) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        onOpenChange(!open);
      } else if (e.key === "Escape" && open) {
        onOpenChange(false);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onOpenChange]);

  if (!open) return null;
  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/45 px-3 pt-20 backdrop-blur-xs"
      onClick={() => onOpenChange(false)}
    >
      <div
        className="w-full max-w-lg overflow-hidden rounded-xl border border-border bg-popover shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <Command className="flex flex-col">
          <div className="flex items-center gap-2 border-b border-border px-3.5 py-2.5">
            <Search className="h-4 w-4 text-muted-foreground" />
            <Command.Input
              autoFocus
              placeholder="搜索来源、命令或跳转页面（Esc 关闭）"
              className="w-full bg-transparent text-sm outline-none placeholder:text-muted-foreground"
            />
          </div>
          <Command.List className="max-h-80 overflow-y-auto p-2 text-sm">
            <Command.Empty className="py-6 text-center text-muted-foreground">未找到匹配项</Command.Empty>
            <Command.Group heading="导航" className="px-2 py-1 text-xs font-medium text-muted-foreground">
              <Command.Item
                onSelect={() => {
                  onSelectTab("sources");
                  onOpenChange(false);
                }}
                className="flex cursor-pointer items-center gap-2 rounded-lg px-2.5 py-2 aria-selected:bg-accent"
              >
                <Database className="h-4 w-4" /> 数据源
              </Command.Item>
              <Command.Item
                onSelect={() => {
                  onSelectTab("export");
                  onOpenChange(false);
                }}
                className="flex cursor-pointer items-center gap-2 rounded-lg px-2.5 py-2 aria-selected:bg-accent"
              >
                <Download className="h-4 w-4" /> 导出队列
              </Command.Item>
            </Command.Group>

            {sources.length > 0 && (
              <Command.Group heading="切换来源" className="mt-1 px-2 py-1 text-xs font-medium text-muted-foreground">
                <Command.Item
                  value="all 全部工具"
                  onSelect={() => {
                    onSelectSource("all");
                    onOpenChange(false);
                  }}
                  className="flex cursor-pointer items-center gap-2 rounded-lg px-2.5 py-2 aria-selected:bg-accent"
                >
                  <Layers className="h-4 w-4 text-muted-foreground" />
                  <span className="truncate">全部工具</span>
                </Command.Item>
                {sources.map((s) => {
                  const label = resolveSourceLabel(s.id, labelMap);
                  return (
                    <Command.Item
                      key={s.id}
                      value={`${label} ${s.id}`}
                      onSelect={() => {
                        onSelectSource(s.id);
                        onOpenChange(false);
                      }}
                      className="flex cursor-pointer items-center gap-2 rounded-lg px-2.5 py-2 aria-selected:bg-accent"
                    >
                      <SourceIcon source={s.id} size={15} />
                      <span className="min-w-0 flex-1 truncate">{label}</span>
                      <span className="shrink-0 font-mono text-[11px] text-muted-foreground">
                        {s.sessionCount}
                      </span>
                    </Command.Item>
                  );
                })}
              </Command.Group>
            )}

            <Command.Group heading="快捷操作" className="mt-1 px-2 py-1 text-xs font-medium text-muted-foreground">
              <Command.Item
                onSelect={() => {
                  onToggleTheme();
                  onOpenChange(false);
                }}
                className="flex cursor-pointer items-center gap-2 rounded-lg px-2.5 py-2 aria-selected:bg-accent"
              >
                {dark ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />} 切换明暗主题
              </Command.Item>
            </Command.Group>
          </Command.List>
        </Command>
      </div>
    </div>
  );
}
