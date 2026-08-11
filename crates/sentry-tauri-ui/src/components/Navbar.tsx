import { invoke } from "@tauri-apps/api/core";
import closeIcon from "/main-navbuttons/close.svg";
import minimizeIcon from "/main-navbuttons/minimize.svg";
import maximizeIcon from "/main-navbuttons/maximize.svg";
import sentryLogo from "/sentry-logo.svg";
import { TABS } from "../config/tabs";
import { useModalStore } from "../store/modal";
import { useTabStore } from "../store/tabs";
import {
  Menubar,
  MenubarContent,
  MenubarItem,
  MenubarMenu,
  MenubarSeparator,
  MenubarTrigger,
} from "./ui/menubar";

function Navbar() {
  const openTab = useTabStore((state) => state.openTab);
  const isModalOpen = useModalStore((state) => state.isOpen);

  return (
    <div className="flex h-10 w-full items-center bg-canvas border-b border-teal select-none">
      <div
        data-tauri-drag-region
        onDoubleClick={() => invoke("maximize_window")}
        className="flex h-full items-center px-3"
      >
        <img
          src={sentryLogo}
          alt="Sentry"
          className={`h-4 w-auto pointer-events-none transition-opacity ${isModalOpen ? "opacity-30" : ""}`}
        />
      </div>
      <div
        className={`flex h-full items-center transition-opacity ${isModalOpen ? "pointer-events-none opacity-30" : ""}`}
      >
        <Menubar className="border-none bg-transparent shadow-none">
          <MenubarMenu>
            <MenubarTrigger className="text-navy">File</MenubarTrigger>
            <MenubarContent>
              <MenubarItem>New Window</MenubarItem>
              <MenubarItem>Open Recent</MenubarItem>
              <MenubarSeparator />
              <MenubarItem>Settings</MenubarItem>
            </MenubarContent>
          </MenubarMenu>
          <MenubarMenu>
            <MenubarTrigger className="text-navy">Edit</MenubarTrigger>
            <MenubarContent>
              <MenubarItem>Undo</MenubarItem>
              <MenubarItem>Redo</MenubarItem>
              <MenubarSeparator />
              <MenubarItem>Preferences</MenubarItem>
            </MenubarContent>
          </MenubarMenu>
          <MenubarMenu>
            <MenubarTrigger className="text-navy">View</MenubarTrigger>
            <MenubarContent>
              {TABS.filter((t) => t.showInViewMenu).map(({ type, defaultId, defaultTitle }) => (
                <MenubarItem
                  key={type}
                  onClick={() =>
                    openTab({ id: defaultId, type, title: defaultTitle })
                  }
                >
                  {defaultTitle}
                </MenubarItem>
              ))}
            </MenubarContent>
          </MenubarMenu>
        </Menubar>
      </div>
      <div
        data-tauri-drag-region
        onDoubleClick={() => invoke("maximize_window")}
        className="h-full flex-1"
      />
      <div className="flex h-full">
        <button
          onClick={() => invoke("minimize_window")}
          className="flex h-full w-11 items-center justify-center text-navy hover:bg-teal/25 active:bg-teal/40"
        >
          <img src={minimizeIcon} alt="Minimize" className="h-3.5 w-3.5" />
        </button>
        <button
          onClick={() => invoke("maximize_window")}
          className="flex h-full w-11 items-center justify-center text-navy hover:bg-teal/25 active:bg-teal/40"
        >
          <img src={maximizeIcon} alt="Maximize" className="h-3.5 w-3.5" />
        </button>
        <button
          onClick={() => invoke("close_window")}
          className="flex h-full w-11 items-center justify-center text-navy hover:bg-danger hover:text-canvas active:bg-danger/85"
        >
          <img src={closeIcon} alt="Close" className="h-3.5 w-3.5" />
        </button>
      </div>
    </div>
  );
}

export default Navbar;
