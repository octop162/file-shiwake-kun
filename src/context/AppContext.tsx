/**
 * アプリケーショングローバルステート管理
 * Global state management using React Context API
 */

import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { Config, ProcessResult } from '../types';
import { loadConfig, saveConfig } from '../api/tauri';

/**
 * アプリケーションステートの型定義
 */
interface AppState {
  config: Config | null;
  isLoading: boolean;
  error: string | null;
  processResults: ProcessResult[];
  isProcessing: boolean;
}

/**
 * コンテキストの型定義
 */
interface AppContextType extends AppState {
  updateConfig: (config: Config) => Promise<void>;
  setProcessResults: (results: ProcessResult[]) => void;
  setIsProcessing: (isProcessing: boolean) => void;
  clearError: () => void;
  reloadConfig: () => Promise<void>;
}

/**
 * デフォルトのコンテキスト値
 */
const defaultContext: AppContextType = {
  config: null,
  isLoading: false,
  error: null,
  processResults: [],
  isProcessing: false,
  updateConfig: async () => {},
  setProcessResults: () => {},
  setIsProcessing: () => {},
  clearError: () => {},
  reloadConfig: async () => {},
};

const AppContext = createContext<AppContextType>(defaultContext);

/**
 * カスタムフック: アプリケーションコンテキストを使用
 */
export const useApp = () => {
  const context = useContext(AppContext);
  if (!context) {
    throw new Error('useApp must be used within AppProvider');
  }
  return context;
};

/**
 * アプリケーションコンテキストプロバイダー
 */
interface AppProviderProps {
  children: ReactNode;
}

export const AppProvider: React.FC<AppProviderProps> = ({ children }) => {
  const [config, setConfig] = useState<Config | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [processResults, setProcessResults] = useState<ProcessResult[]>([]);
  const [isProcessing, setIsProcessing] = useState<boolean>(false);

  /**
   * 初期化時に設定を読み込む
   */
  useEffect(() => {
    loadInitialConfig();
  }, []);

  /**
   * 設定の初期読み込み
   */
  const loadInitialConfig = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const loadedConfig = await loadConfig();
      setConfig(loadedConfig);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load configuration';
      setError(errorMessage);
      console.error('Failed to load config:', err);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * 設定を更新して保存
   */
  const updateConfig = async (newConfig: Config) => {
    setError(null);
    try {
      await saveConfig(newConfig);
      setConfig(newConfig);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to save configuration';
      setError(errorMessage);
      console.error('Failed to save config:', err);
      throw err;
    }
  };

  /**
   * 設定を再読み込み
   */
  const reloadConfig = async () => {
    await loadInitialConfig();
  };

  /**
   * エラーをクリア
   */
  const clearError = () => {
    setError(null);
  };

  const value: AppContextType = {
    config,
    isLoading,
    error,
    processResults,
    isProcessing,
    updateConfig,
    setProcessResults,
    setIsProcessing,
    clearError,
    reloadConfig,
  };

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
};
