import AppLogo from "./components/AppLogo";
import ModalSocket from "./components/ModalSocket";
import Navbar from "./components/Navbar";
import Sidebar from "./components/Sidebar";
import SidePanel from "./components/SidePanel";
import TabBar from "./components/TabBar";
import TabContent from "./components/TabContent";

function App() {
  return (
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
  );
}

export default App;
