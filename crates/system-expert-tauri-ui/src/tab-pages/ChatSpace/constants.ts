import { Activity, Cpu, HardDrive, MemoryStick } from "lucide-react";
import type { ComponentType } from "react";

export type ModelId = "qwen" | "gpt-oss";

export interface ModelOption {
  id: ModelId;
  label: string;
  /** One-line positioning shown under the label in the model picker. */
  hint: string;
}

export const MODELS: ModelOption[] = [
  { id: "qwen", label: "Qwen", hint: "Fast answers on live metrics" },
  { id: "gpt-oss", label: "GPT-OSS", hint: "Slower, deeper reasoning" },
];

export const MODEL_LABELS: Record<ModelId, string> = {
  qwen: "Qwen",
  "gpt-oss": "GPT-OSS",
};

export interface SuggestedPrompt {
  icon: ComponentType<{ className?: string }>;
  label: string;
  /** The text actually sent — the label is the short, scannable version. */
  prompt: string;
}

/** Shown on an empty chat so the first message is one click away rather than a
 * blank page the user has to guess at. */
export const SUGGESTED_PROMPTS: SuggestedPrompt[] = [
  {
    icon: Cpu,
    label: "What's eating my CPU?",
    prompt: "Which processes are using the most CPU right now?",
  },
  {
    icon: MemoryStick,
    label: "Check memory pressure",
    prompt: "How much memory is in use, and which apps are the biggest consumers?",
  },
  {
    icon: HardDrive,
    label: "Find heavy disk activity",
    prompt: "What is reading and writing to disk the most right now?",
  },
  {
    icon: Activity,
    label: "Summarize system health",
    prompt: "Give me an overall health summary of my system.",
  },
];
