import { Routes, Route, Navigate } from "react-router";
import Layout from "./components/layout/Layout";
import Dashboard from "./pages/Dashboard";
import Mods from "./pages/Mods";
import Translation from "./pages/Translation";
import Editor from "./pages/Editor";
import Settings from "./pages/Settings";
import Stats from "./pages/Stats";
import History from "./pages/History";
import Diagnostics from "./pages/Diagnostics";

export default function App() {
  return (
    <Layout>
      <Routes>
        <Route path="/" element={<Navigate to="/dashboard" replace />} />
        <Route path="/dashboard" element={<Dashboard />} />
        <Route path="/mods" element={<Mods />} />
        <Route path="/translation" element={<Translation />} />
        <Route path="/editor" element={<Editor />} />
        <Route path="/stats" element={<Stats />} />
        <Route path="/history" element={<History />} />
        <Route path="/diagnostics" element={<Diagnostics />} />
        <Route path="/settings" element={<Settings />} />
      </Routes>
    </Layout>
  );
}