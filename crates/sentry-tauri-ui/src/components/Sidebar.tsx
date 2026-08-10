import activityMonitorIcon from "/prim-sidebar-buttons/activity-monitor.svg";
import chatbotIcon from "/prim-sidebar-buttons/chatbot.svg";
import detailsViewIcon from "/prim-sidebar-buttons/details-view.svg";

function Sidebar() {
  return (
    <div className="flex h-full w-12 flex-col items-center bg-[#f5f2ea] border-r border-[#e5e0d3] select-none">
      <button
        onClick={() => {}}
        className="flex h-12 w-12 items-center justify-center text-neutral-600 hover:bg-black/5 active:bg-black/10"
      >
        <img src={activityMonitorIcon} alt="Activity Monitor" className="h-5 w-5" />
      </button>
      <button
        onClick={() => {}}
        className="flex h-12 w-12 items-center justify-center text-neutral-600 hover:bg-black/5 active:bg-black/10"
      >
        <img src={chatbotIcon} alt="Chatbot" className="h-5 w-5" />
      </button>
      <button
        onClick={() => {}}
        className="flex h-12 w-12 items-center justify-center text-neutral-600 hover:bg-black/5 active:bg-black/10"
      >
        <img src={detailsViewIcon} alt="Details View" className="h-5 w-5" />
      </button>
    </div>
  );
}

export default Sidebar;
