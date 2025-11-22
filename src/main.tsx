/**
 * アプリケーションエントリーポイント
 * Application entry point with context providers
 */

import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { AppProvider, ViewProvider } from "./context";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <AppProvider>
      <ViewProvider>
        <App />
      </ViewProvider>
    </AppProvider>
  </React.StrictMode>,
);
