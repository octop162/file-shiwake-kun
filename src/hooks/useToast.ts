/**
 * トースト通知カスタムフック
 * Custom hook for managing toast notifications
 */

import { useState, useCallback } from 'react';
import { ToastMessage, ToastType } from '../components/Toast';

let toastIdCounter = 0;

export const useToast = () => {
  const [toasts, setToasts] = useState<ToastMessage[]>([]);

  const addToast = useCallback((
    message: string,
    type: ToastType = ToastType.Info,
    duration?: number
  ) => {
    const id = `toast-${++toastIdCounter}`;
    const newToast: ToastMessage = {
      id,
      message,
      type,
      duration,
    };

    setToasts((prev) => [...prev, newToast]);
    return id;
  }, []);

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((toast) => toast.id !== id));
  }, []);

  const success = useCallback((message: string, duration?: number) => {
    return addToast(message, ToastType.Success, duration);
  }, [addToast]);

  const error = useCallback((message: string, duration?: number) => {
    return addToast(message, ToastType.Error, duration);
  }, [addToast]);

  const warning = useCallback((message: string, duration?: number) => {
    return addToast(message, ToastType.Warning, duration);
  }, [addToast]);

  const info = useCallback((message: string, duration?: number) => {
    return addToast(message, ToastType.Info, duration);
  }, [addToast]);

  const clearAll = useCallback(() => {
    setToasts([]);
  }, []);

  return {
    toasts,
    addToast,
    removeToast,
    success,
    error,
    warning,
    info,
    clearAll,
  };
};
