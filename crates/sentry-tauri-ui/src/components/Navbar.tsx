import { invoke } from "@tauri-apps/api/core";
import closeIcon from "/main-navbuttons/close.svg";
import minimizeIcon from "/main-navbuttons/minimize.svg";
import maximizeIcon from "/main-navbuttons/maximize.svg";

function Navbar() {
  return (
    <div className="flex h-10 w-full items-center bg-[#f5f2ea] border-b border-[#e5e0d3] select-none">
      <div
        data-tauri-drag-region
        onDoubleClick={() => invoke("maximize_window")}
        className="flex h-full flex-1 items-center px-3"
      >
        <span className="text-sm font-medium text-neutral-700 pointer-events-none">
          Sentry
        </span>
      </div>
      <div className="flex h-full">
        <button
          onClick={() => invoke("minimize_window")}
          className="flex h-full w-11 items-center justify-center text-neutral-600 hover:bg-black/5 active:bg-black/10"
        >
          <img src={minimizeIcon} alt="Minimize" className="h-3.5 w-3.5" />
        </button>
        <button
          onClick={() => invoke("maximize_window")}
          className="flex h-full w-11 items-center justify-center text-neutral-600 hover:bg-black/5 active:bg-black/10"
        >
          <img src={maximizeIcon} alt="Maximize" className="h-3.5 w-3.5" />
        </button>
        <button
          onClick={() => invoke("close_window")}
          className="flex h-full w-11 items-center justify-center text-neutral-600 hover:bg-red-500 hover:text-white active:bg-red-600"
        >
          <img src={closeIcon} alt="Close" className="h-3.5 w-3.5" />
        </button>
      </div>
    </div>
  );
}

export default Navbar;
