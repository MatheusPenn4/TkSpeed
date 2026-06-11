import React from "react";
import ReactDOM from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { App } from "./App";
import "./styles/tokens.css";
import "./styles/global.css";
import "./shared/components/ui.css";
// Design System V3 "Apex" — fontes Geist self-hosted (offline) + tokens.
import "@fontsource/geist-sans/latin-400.css";
import "@fontsource/geist-sans/latin-500.css";
import "@fontsource/geist-sans/latin-600.css";
import "@fontsource/geist-mono/latin-400.css";
import "@fontsource/geist-mono/latin-500.css";
import "@fontsource/geist-mono/latin-600.css";
import "./shared/apex/apex.css";

const queryClient = new QueryClient();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <App />
    </QueryClientProvider>
  </React.StrictMode>,
);
