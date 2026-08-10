import { create } from "zustand";
import type { ReactNode } from "react";

interface ModalStore {
  content: ReactNode | null;
  isOpen: boolean;
  openModal: (content: ReactNode) => void;
  closeModal: () => void;
  /** Clears the content once the close transition has finished. */
  clearContent: () => void;
}

export const useModalStore = create<ModalStore>((set) => ({
  content: null,
  isOpen: false,
  openModal: (content) => set({ content, isOpen: true }),
  closeModal: () => set({ isOpen: false }),
  clearContent: () => set({ content: null }),
}));
