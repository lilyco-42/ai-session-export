export type SourceId = string;
export type ToolId = SourceId;
export type BuiltInSourceId =
  | "codearts"
  | "opencode"
  | "agy"
  | "codex"
  | "dsh"
  | "claude"
  | "gemini"
  | "pi";
export type SourceStatus = "ready" | "missing" | "error";
export type ExportFormat = "markdown" | "jsonl" | "raw" | "html";

export interface SourceSummary {
  id: SourceId;
  label: string;
  status: SourceStatus;
  sessionCount: number;
}

export interface SessionSummary {
  id: string;
  source: SourceId;
  title: string;
  updatedAt: string;
}

export interface ApiErrorShape {
  error_code: string;
  reason_zh: string;
  reason_en: string;
  repair_zh: string;
}

function isRecord(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null;
}

function asSourceId(v: unknown): SourceId | null {
  return typeof v === "string" && v.trim().length > 0 ? v.trim() : null;
}

function asSource(v: unknown): SourceSummary | null {
  if (!isRecord(v)) return null;
  const id = asSourceId(v.id);
  const { label, status, sessionCount } = v;
  if (!id || typeof label !== "string") return null;
  if (status !== "ready" && status !== "missing" && status !== "error") return null;
  if (typeof sessionCount !== "number" || !Number.isFinite(sessionCount) || sessionCount < 0) return null;
  return { id, label: label.trim() || id, status, sessionCount };
}

function asSession(v: unknown): SessionSummary | null {
  if (!isRecord(v)) return null;
  const source = asSourceId(v.source);
  const { id, title, updatedAt } = v;
  if (!source || typeof id !== "string" || typeof title !== "string" || typeof updatedAt !== "string") return null;
  return { id, source, title, updatedAt };
}

function asError(v: unknown): ApiErrorShape | null {
  if (!isRecord(v)) return null;
  const keys: (keyof ApiErrorShape)[] = ["error_code", "reason_zh", "reason_en", "repair_zh"];
  if (!keys.every((k) => typeof v[k] === "string")) return null;
  return v as unknown as ApiErrorShape;
}

export class ApiError extends Error {
  shape: ApiErrorShape;
  constructor(shape: ApiErrorShape) {
    super(shape.reason_zh);
    this.shape = shape;
  }
}

async function readJson(path: string, init?: RequestInit): Promise<unknown> {
  const res = await fetch(path, init);
  const raw: unknown = await res.json().catch(() => null);
  if (!res.ok) {
    const shape = asError(raw);
    throw new ApiError(
      shape ?? {
        error_code: "E_API_STATUS",
        reason_zh: "接口返回异常状态",
        reason_en: "API returned an abnormal status",
        repair_zh: "请检查本地导出服务是否已启动。",
      },
    );
  }
  return raw;
}

export async function loadSources(): Promise<SourceSummary[]> {
  const raw: unknown = await readJson("/api/sources");
  if (!Array.isArray(raw)) {
    throw new ApiError({
      error_code: "E_SOURCES_SHAPE",
      reason_zh: "数据源列表格式不正确",
      reason_en: "Source list has an invalid shape",
      repair_zh: "请确认后端返回 SourceSummary 数组。",
    });
  }
  return raw.map(asSource).filter((x): x is SourceSummary => x !== null);
}

export async function loadSessions(source: SourceId | "all"): Promise<SessionSummary[]> {
  const raw: unknown = await readJson("/api/sessions?source=" + encodeURIComponent(source));
  if (!Array.isArray(raw)) {
    throw new ApiError({
      error_code: "E_SESSIONS_SHAPE",
      reason_zh: "会话列表格式不正确",
      reason_en: "Session list has an invalid shape",
      repair_zh: "请确认后端返回 SessionSummary 数组。",
    });
  }
  return raw.map(asSession).filter((x): x is SessionSummary => x !== null);
}

export async function exportSessions(ids: string[], format: ExportFormat): Promise<{ downloadUrl: string }> {
  const raw: unknown = await readJson("/api/export", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ ids, format }),
  });
  if (!isRecord(raw) || typeof raw.downloadUrl !== "string") {
    throw new ApiError({
      error_code: "E_EXPORT_SHAPE",
      reason_zh: "导出结果格式不正确",
      reason_en: "Export result has an invalid shape",
      repair_zh: "请确认后端返回 downloadUrl 字段。",
    });
  }
  return { downloadUrl: raw.downloadUrl };
}
