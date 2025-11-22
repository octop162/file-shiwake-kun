/**
 * ビュー管理コンテキスト
 * View management context for simple routing
 */

import React, { createContext, useContext, useState, ReactNode } from 'react';

/**
 * アプリケーションのビュー
 */
export enum View {
  Main = 'main',
  Settings = 'settings',
  Results = 'results',
}

/**
 * ビューコンテキストの型定義
 */
interface ViewContextType {
  currentView: View;
  setView: (view: View) => void;
  goToMain: () => void;
  goToSettings: () => void;
  goToResults: () => void;
}

const ViewContext = createContext<ViewContextType | undefined>(undefined);

/**
 * カスタムフック: ビューコンテキストを使用
 */
export const useView = () => {
  const context = useContext(ViewContext);
  if (!context) {
    throw new Error('useView must be used within ViewProvider');
  }
  return context;
};

/**
 * ビューコンテキストプロバイダー
 */
interface ViewProviderProps {
  children: ReactNode;
}

export const ViewProvider: React.FC<ViewProviderProps> = ({ children }) => {
  const [currentView, setCurrentView] = useState<View>(View.Main);

  const setView = (view: View) => {
    setCurrentView(view);
  };

  const goToMain = () => {
    setCurrentView(View.Main);
  };

  const goToSettings = () => {
    setCurrentView(View.Settings);
  };

  const goToResults = () => {
    setCurrentView(View.Results);
  };

  const value: ViewContextType = {
    currentView,
    setView,
    goToMain,
    goToSettings,
    goToResults,
  };

  return <ViewContext.Provider value={value}>{children}</ViewContext.Provider>;
};
