import { useEffect } from "react";
import { Command } from "cmdk";
import { Database, Download, Moon, Search, Sun } from "lucide-react";

interface Props {
  open: boolean;
  onOpenChange: (v: boolean) => void;
  onSelectTab: (id: "sources" | "export") => void;
  onToggleTheme: () => void;
  dark: boolean;
}

export function CommandMenu({ open, onOpenChange, onSelectTab, onToggleTheme, dark }: Props) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        onOpenChange(!open);
      } else if (e.key === "Escape" && open) onOpenChange(false);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onOpenChange]);

  if (!open) return null;
  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center bg-black/45 px-3 pt-24 backdrop-blur-xs" onClick={() => onOpenChange(false)}>
      <div className="w-full max-w-lg overflow-hidden rounded-xl border border-border bg-popover shadow-2xl" onClick={(e) => e.stopPropagation()}>
        <Command className="flex flex-col">
          <div className="flex items-center gap-2 border-b border-border px-3.5 py-2.5">
            <Search className="h-4 w-4 text-muted-foreground" />
            <Command.Input autoFocus placeholder="输入命令或跳转页面（Esc 关闭）" className="w-full bg-transparent text-sm outline-none placeholder:text-muted-foreground" />
          </div>
          <Command.List className="max-h-72 overflow-y-auto p-2 text-sm">
            <Command.Empty className="py-6 text-center text-muted-foreground">未找到匹配项</Command.Empty>
            <Command.Group heading="导航" className="px-2 py-1 text-xs font-medium text-muted-foreground">
              <Command.Item onSelect={() => { onSelectTab("sources"); onOpenChange(false); }} className="flex cursor-pointer items-center gap-2 rounded-lg px-2.5 py-2 aria-selected:bg-accent">
                <Database className="h-4 w-4" /> 数据源
              </Command.Item>
              <Command.Item onSelect={() => { onSelectTab("export"); onOpenChange(false); }} className="flex cursor-pointer items-center gap-2 rounded-lg px-2.5 py-2 aria-selected:bg-accent">
                <Download className="h-4 w-4" /> 导出队列
              </Command.Item>
            </Command.Group>
            <Command.Group heading="快捷操作" className="mt-1 px-2 py-1 text-xs font-medium text-muted-foreground">
              <Command.Item onSelect={() => { onToggleTheme(); onOpenChange(false); }} className="flex cursor-pointer items-center gap-2 rounded-lg px-2.5 py-2 aria-selected:bg-accent">
                {dark ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />} 切换明暗主题
              </Command.Item>
            </Command.Group>
          </Command.List>
        </Command>
      </div>
    </div>
  );
}
