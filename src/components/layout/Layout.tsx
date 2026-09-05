import { ReactNode } from "react";
import Sidebar from "../ui/Sidebar";
import TopBar from "./TopBar";
import ParticleBackground from "../particles/ParticleBackground";

export default function Layout({ children }: { children: ReactNode }) {
  return (
    <div className="flex h-screen w-screen overflow-hidden relative">
      <ParticleBackground />
      <Sidebar />
      <div className="flex-1 flex flex-col relative z-10">
        <TopBar />
        <main className="flex-1 overflow-auto p-6">{children}</main>
      </div>
    </div>
  );
}