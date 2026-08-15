import { useEffect } from "react";
import AppLogo from "./components/AppLogo";
import ModalSocket from "./components/ModalSocket";
import Navbar from "./components/Navbar";
import Sidebar from "./components/Sidebar";
import SidePanel from "./components/SidePanel";
import TabBar from "./components/TabBar";
import TabContent from "./components/TabContent";
import { TooltipProvider } from "./components/ui/tooltip";
import { watchAgentStatus } from "./store/agent";

function App() {
  // Driven by the agent's WebSocket connecting and disconnecting — no polling.
  useEffect(() => watchAgentStatus(), []);

  return (
    <TooltipProvider delay={400}>
      <div className="flex h-screen w-screen flex-col">
        <Navbar />
        <div className="relative bg-dot-pattern flex flex-1 w-full overflow-hidden">
          <Sidebar />
          <SidePanel />
          <div className="relative flex flex-1 flex-col overflow-hidden">
            <AppLogo />
            <TabBar />
            <TabContent />
          </div>
          <ModalSocket />
        </div>
      </div>
    </TooltipProvider>
  );
}

export default App;
