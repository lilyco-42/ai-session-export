import { useEffect, useState } from "react";
import { loadSessionDetail, loadTaskStatus, loadResumePlan, type SessionDetail, type TaskObservation, type ResumePlan } from "@/api";
import { Button } from "@/components/ui/button";

const STATUS = { unknown: "未知", started: "日志记录已开始（不代表正在运行）", completed: "日志记录已完成", aborted: "日志记录已中断" };

export function SessionPreview({ id, onClose }: { id: string; onClose: () => void }) {
  const [detail, setDetail] = useState<SessionDetail | null>(null);
  const [task, setTask] = useState<TaskObservation | null>(null);
  const [plan, setPlan] = useState<ResumePlan | null>(null);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  useEffect(() => {
    let active = true;
    loadSessionDetail(id).then(v => { if (active) setDetail(v); }).catch(() => { if (active) setError("读取对话失败。"); });
    loadTaskStatus(id).then(v => { if (active) setTask(v); }).catch(() => { if (active) setError("任务状态读取失败。"); });
    loadResumePlan(id).then(v => { if (active) setPlan(v); }).catch(() => { if (active) setError("恢复方案读取失败。"); });
    return () => { active = false; };
  }, [id]);
  async function more() {
    if (detail?.nextOffset == null || loading) return;
    setLoading(true);
    try {
      const next = await loadSessionDetail(id, detail.nextOffset);
      setDetail({ ...next, messages: [...detail.messages, ...next.messages] });
    } catch { setError("读取下一页失败。"); }
    finally { setLoading(false); }
  }
  return (
    <section className="space-y-3 border-t border-border p-4" aria-label="会话详情">
      <div className="flex items-center justify-between gap-2">
        <h2 className="font-semibold break-all">{detail?.session.title ?? "读取对话..."}</h2>
        <Button variant="outline" size="sm" onClick={onClose}>关闭</Button>
      </div>
      {error && <p role="alert" className="text-sm text-destructive">{error}</p>}
      {task && <div className="rounded-md border p-3 text-sm">
        <p>任务状态：{STATUS[task.status] ?? task.status}</p>
        <p className="text-xs text-muted-foreground">仅日志观测，不代表实时进程。{task.observedAt ?? ""}</p>
      </div>}
      {plan && <div className="rounded-md border p-3 text-sm">
        {plan.supported ? <><p>在原 Agent 环境手动执行：</p><pre className="overflow-auto whitespace-pre-wrap break-all">{plan.argv.join(" ")}</pre></> : <p>此来源不支持原生恢复，可导出上下文继续。</p>}
      </div>}
      {detail && <>
        <p className="text-xs text-muted-foreground">已显示 {detail.messages.length} / {detail.totalMessages} 条消息</p>
        <div className="max-h-[480px] space-y-3 overflow-y-auto">
          {detail.messages.map((msg, i) => <article className="rounded-md border p-3" key={i}>
            <p className="mb-1 text-xs font-semibold text-muted-foreground">{msg.role}</p>
            <pre className="whitespace-pre-wrap break-words text-sm">{msg.text}</pre>
          </article>)}
        </div>
        {detail.nextOffset !== null && <Button size="sm" onClick={more} disabled={loading}>加载更多</Button>}
      </>}
    </section>
  );
}
