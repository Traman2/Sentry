import { useRef } from "react";
import {
  Dialog,
  DialogBackdrop,
  DialogPopup,
  DialogPortal,
} from "./ui/dialog";
import { useModalStore } from "../store/modal";

/**
 * Mounted once at the app root, scoped to the content area (a sibling of the
 * window title bar). Any component can call `useModalStore().openModal(node)`
 * to render `node` here, dimmed on top of the rest of the app. The socket
 * owns positioning/dimming/open-close only — each modal's content owns its
 * own card styling.
 *
 * `modal="trap-focus"` (not `true`) is required: base-ui's default modal
 * backdrop is `position: fixed; inset: 0`, which would cover the whole
 * viewport including the title bar's minimize/maximize/close buttons.
 * `"trap-focus"` keeps focus trapping/Escape/outside-press behavior without
 * rendering that viewport-covering backdrop.
 */
function ModalSocket() {
  const containerRef = useRef<HTMLDivElement>(null);
  const isOpen = useModalStore((state) => state.isOpen);
  const content = useModalStore((state) => state.content);
  const closeModal = useModalStore((state) => state.closeModal);
  const clearContent = useModalStore((state) => state.clearContent);

  return (
    <div ref={containerRef} className="pointer-events-none absolute inset-0">
      <Dialog
        open={isOpen}
        modal="trap-focus"
        onOpenChange={(open) => {
          if (!open) closeModal();
        }}
        onOpenChangeComplete={(open) => {
          if (!open) clearContent();
        }}
      >
        <DialogPortal container={containerRef}>
          <DialogBackdrop className="pointer-events-auto absolute inset-0 bg-black/60" />
          <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
            <DialogPopup className="pointer-events-auto outline-none">
              {content}
            </DialogPopup>
          </div>
        </DialogPortal>
      </Dialog>
    </div>
  );
}

export default ModalSocket;
