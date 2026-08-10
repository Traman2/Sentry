import { HashRouter, Route, Routes } from "react-router-dom";
import Navbar from "./components/Navbar";
import Sidebar from "./components/Sidebar";
import Home from "./pages/Home";

function App() {
  return (
    <div className="flex h-screen w-screen flex-col">
      <Navbar />
      <div className="flex flex-1 w-full overflow-hidden">
        <Sidebar />
        <main className="bg-dot-pattern flex-1 overflow-auto">
          <HashRouter>
            <Routes>
              <Route path="/" element={<Home />} />
            </Routes>
          </HashRouter>
        </main>
      </div>
    </div>
  );
}

export default App;
