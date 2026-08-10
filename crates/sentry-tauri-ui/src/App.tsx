import Navbar from "./components/Navbar";
import Sidebar from "./components/Sidebar";
import SidePanel from "./components/SidePanel";
import TabBar from "./components/TabBar";
import TabContent from "./components/TabContent";

function App() {
  return (
    <div className="flex h-screen w-screen flex-col">
      <Navbar />
      <div className="bg-dot-pattern flex flex-1 w-full overflow-hidden">
        <Sidebar />
        <SidePanel />
        <div className="flex flex-1 flex-col overflow-hidden">
          <TabBar />
          <TabContent />
        </div>
      </div>
    </div>
  );
}

export default App;
