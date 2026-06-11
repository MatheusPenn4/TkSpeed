import { ReactNode } from "react";
import { Sidebar } from "./Sidebar";
import { TitleBar } from "./TitleBar";
import "./shell.css";

export function AppShell({ children }: { children: ReactNode }) {
  return (
    <div className="shell">
      <TitleBar />
      <div className="shell-body">
        <Sidebar />
        <main className="shell-content">{children}</main>
      </div>
    </div>
  );
}
