/**
 * アプリケーションエントリーポイント
 * Application entry point with context providers
 */

import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { AppProvider, ViewProvider } from "./context";
import { ErrorBoundary } from "./components";
import "./index.css";

// Initialize dark mode based on system preference
if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
  document.documentElement.classList.add('dark');
} else {
  document.documentElement.classList.remove('dark');
}

// Listen for system theme changes
window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
  if (e.matches) {
    document.documentElement.classList.add('dark');
  } else {
    document.documentElement.classList.remove('dark');
  }
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ErrorBoundary>
      <AppProvider>
        <ViewProvider>
          <App />
        </ViewProvider>
      </AppProvider>
    </ErrorBoundary>
  </React.StrictMode>,
);
