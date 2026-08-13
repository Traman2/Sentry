import { openUrl } from "@tauri-apps/plugin-opener";
import { memo } from "react";
import ReactMarkdown, { type Components } from "react-markdown";
import remarkGfm from "remark-gfm";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { cn } from "@/lib/utils";

/** Agent replies arrive as markdown, so every element gets an explicit class
 * rather than a `prose` preset: the app ships one light theme built on the
 * brand tokens, and a typography plugin would drag in its own gray scale.
 *
 * Spacing is expressed as `not-first:mt-*` so a block never adds a leading gap
 * — the transcript already spaces turns, and a stray top margin would push the
 * first line away from the row above it. */
const components: Components = {
  p: ({ className, ...props }) => (
    <p className={cn("not-first:mt-3 leading-relaxed wrap-break-word", className)} {...props} />
  ),

  // Replies rarely need a document hierarchy, so the headings converge on
  // roughly the body size and lean on weight and color for rank instead.
  h1: ({ className, ...props }) => (
    <h1
      className={cn("not-first:mt-5 text-base font-semibold text-navy", className)}
      {...props}
    />
  ),
  h2: ({ className, ...props }) => (
    <h2
      className={cn("not-first:mt-5 text-[15px] font-semibold text-navy", className)}
      {...props}
    />
  ),
  h3: ({ className, ...props }) => (
    <h3 className={cn("not-first:mt-4 text-sm font-semibold text-navy", className)} {...props} />
  ),
  h4: ({ className, ...props }) => (
    <h4
      className={cn("not-first:mt-4 text-sm font-medium text-navy/80", className)}
      {...props}
    />
  ),
  h5: ({ className, ...props }) => (
    <h5
      className={cn("not-first:mt-4 text-sm font-medium text-navy/80", className)}
      {...props}
    />
  ),
  h6: ({ className, ...props }) => (
    <h6
      className={cn("not-first:mt-4 text-sm font-medium text-navy/80", className)}
      {...props}
    />
  ),

  ul: ({ className, ...props }) => (
    <ul className={cn("not-first:mt-3 list-disc space-y-1 pl-5", className)} {...props} />
  ),
  ol: ({ className, ...props }) => (
    <ol className={cn("not-first:mt-3 list-decimal space-y-1 pl-5", className)} {...props} />
  ),
  li: ({ className, ...props }) => (
    // Nested lists sit inside an `li`, where the outer `space-y` would leave
    // them flush against their parent line.
    <li className={cn("leading-relaxed [&>ul]:mt-1 [&>ol]:mt-1", className)} {...props} />
  ),

  blockquote: ({ className, ...props }) => (
    <blockquote
      className={cn("not-first:mt-3 border-l-2 border-teal pl-3 text-navy/70", className)}
      {...props}
    />
  ),

  hr: ({ className, ...props }) => (
    <hr className={cn("my-4 border-teal/40", className)} {...props} />
  ),

  strong: ({ className, ...props }) => (
    <strong className={cn("font-semibold text-navy", className)} {...props} />
  ),

  // Inline styling by default; the `pre` below strips it back off for fenced
  // blocks, which avoids guessing at inline-ness from the language class
  // (indented and untagged blocks carry none).
  code: ({ className, ...props }) => (
    <code
      className={cn(
        "rounded-[min(var(--radius-sm),6px)] bg-mint/40 px-1 py-0.5 font-mono text-[0.85em] text-navy",
        className,
      )}
      {...props}
    />
  ),
  pre: ({ className, ...props }) => (
    <pre
      className={cn(
        "not-first:mt-3 overflow-x-auto rounded-lg border border-teal/50 bg-mint/20 p-3 font-mono text-xs leading-relaxed text-navy",
        "[&>code]:block [&>code]:rounded-none [&>code]:bg-transparent [&>code]:p-0 [&>code]:text-xs",
        className,
      )}
      {...props}
    />
  ),

  // Tauri's webview has nowhere to put a new tab, so a plain target=_blank
  // link would dead-end. Hand it to the OS browser instead.
  a: ({ className, href, children, ...props }) => (
    <a
      href={href}
      className={cn("font-medium text-navy underline underline-offset-2 hover:text-navy/70", className)}
      onClick={(event) => {
        if (!href) return;
        event.preventDefault();
        void openUrl(href);
      }}
      {...props}
    >
      {children}
    </a>
  ),

  table: ({ className, ...props }) => (
    <Table
      containerClassName="not-first:mt-3 rounded-lg border border-teal/50"
      className={cn("text-xs", className)}
      {...props}
    />
  ),
  thead: (props) => <TableHeader {...props} />,
  tbody: (props) => <TableBody {...props} />,
  tr: (props) => <TableRow {...props} />,
  th: ({ className, ...props }) => (
    <TableHead className={cn("h-8 text-xs font-semibold text-navy", className)} {...props} />
  ),
  td: ({ className, ...props }) => (
    <TableCell className={cn("py-1.5 align-top", className)} {...props} />
  ),
};

/** Renders one assistant reply. Memoized because the transcript re-renders on
 * every poll and every keystroke in the composer, while a reply's markdown
 * never changes once it has landed. */
export const Markdown = memo(function Markdown({ content }: { content: string }) {
  return (
    <div className="wrap-break-word">
      <ReactMarkdown remarkPlugins={[remarkGfm]} components={components}>
        {content}
      </ReactMarkdown>
    </div>
  );
});
