import { ArrowUp, ChevronDown } from "lucide-react";
import { useEffect, useLayoutEffect, useRef } from "react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { InputGroup, InputGroupAddon, InputGroupTextarea } from "@/components/ui/input-group";
import { Kbd } from "@/components/ui/kbd";
import { Spinner } from "@/components/ui/spinner";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { MODEL_LABELS, MODELS, type ModelId } from "./constants";

/** The composer input pill: textarea on top, controls docked along the bottom
 * edge inside the same bordered surface. */
export function Composer({
  draft,
  onDraftChange,
  onSend,
  model,
  onModelChange,
  disabled,
  sending,
  placeholder,
}: {
  draft: string;
  onDraftChange: (value: string) => void;
  onSend: () => void;
  model: ModelId;
  onModelChange: (model: ModelId) => void;
  disabled: boolean;
  sending: boolean;
  placeholder: string;
}) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Grows the input pill by one line per newline, up to MAX_VISIBLE_LINES, then
  // switches to an internal scrollbar — matches Claude's chat input behavior.
  const resizeTextarea = () => {
    const el = textareaRef.current;
    if (!el) return;
    const MAX_VISIBLE_LINES = 8;
    const lineHeight = parseFloat(getComputedStyle(el).lineHeight) || 20;
    const maxHeight = lineHeight * MAX_VISIBLE_LINES;

    el.style.height = "auto";
    const nextHeight = Math.min(el.scrollHeight, maxHeight);
    el.style.height = `${nextHeight}px`;
    el.style.overflowY = el.scrollHeight > maxHeight ? "auto" : "hidden";
  };

  useLayoutEffect(() => {
    resizeTextarea();
  }, [draft]);

  // The custom web font (Geist Variable) can finish loading after the layout
  // above already ran, which changes line-height and leaves the box sized for
  // the fallback font's smaller metrics — it looks "tiny" until something else
  // (like typing) triggers a resize. Recompute once the real font is ready.
  useEffect(() => {
    document.fonts?.ready.then(() => resizeTextarea());
  }, []);

  const canSend = Boolean(draft.trim()) && !sending && !disabled;

  return (
    <div className="shrink-0 px-4 pt-2 pb-4">
      <div className="mx-auto w-full max-w-3xl">
        <InputGroup className="rounded-2xl border-teal/50 bg-canvas shadow-sm transition-colors focus-within:border-teal">
          <InputGroupTextarea
            ref={textareaRef}
            value={draft}
            onChange={(e) => onDraftChange(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                onSend();
              }
            }}
            placeholder={placeholder}
            rows={1}
            disabled={disabled}
            aria-label="Message System-Expert"
            className="min-h-0 px-3.5 pt-3 pb-1 text-sm leading-relaxed text-navy placeholder:text-navy/40"
          />

          <InputGroupAddon align="block-end" className="border-t border-teal/25">
            <DropdownMenu>
              <DropdownMenuTrigger
                disabled={disabled}
                className="flex h-6 cursor-pointer items-center gap-1 rounded-md px-2 text-[11px] font-medium text-navy/70 transition-colors outline-none select-none hover:bg-teal/25 hover:text-navy focus-visible:ring-3 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-expanded:bg-teal/25"
              >
                {MODEL_LABELS[model]}
                <ChevronDown className="size-3" />
              </DropdownMenuTrigger>
              <DropdownMenuContent align="start" className="w-auto min-w-56">
                <DropdownMenuRadioGroup
                  value={model}
                  onValueChange={(value) => onModelChange(value as ModelId)}
                >
                  {MODELS.map(({ id, label, hint }) => (
                    <DropdownMenuRadioItem key={id} value={id}>
                      <span className="flex flex-col gap-0.5">
                        <span className="text-xs font-medium text-navy">{label}</span>
                        <span className="text-[11px] text-muted-foreground">{hint}</span>
                      </span>
                    </DropdownMenuRadioItem>
                  ))}
                </DropdownMenuRadioGroup>
              </DropdownMenuContent>
            </DropdownMenu>

            <div className="ml-auto flex items-center gap-2">
              <span className="hidden items-center gap-1 text-[11px] text-navy/40 sm:flex">
                <Kbd>Enter</Kbd>
                to send
              </span>
              <Tooltip>
                {/* `aria-disabled`, not `disabled`: InputGroup greys and tints
                    its whole surface via `has-disabled:`, so a natively
                    disabled send button would wash out the entire composer
                    every time the draft is empty. */}
                <TooltipTrigger
                  render={
                    <Button
                      size="icon-sm"
                      aria-label="Send message"
                      className="rounded-lg aria-disabled:pointer-events-none aria-disabled:opacity-50"
                      onClick={onSend}
                      aria-disabled={!canSend}
                    />
                  }
                >
                  {sending ? (
                    <Spinner className="size-4" />
                  ) : (
                    <ArrowUp className="size-4" />
                  )}
                </TooltipTrigger>
                <TooltipContent>Send message</TooltipContent>
              </Tooltip>
            </div>
          </InputGroupAddon>
        </InputGroup>
      </div>
    </div>
  );
}
