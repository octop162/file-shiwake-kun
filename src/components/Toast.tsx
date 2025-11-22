/**
 * トースト通知コンポーネント
 * Toast notification component for user feedback
 * Requirement 8.2: Display clear error messages with reasons
 */

import { useEffect } from 'react';

export enum ToastType {
  Success = 'success',
  Error = 'error',
  Warning = 'warning',
  Info = 'info',
}

export interface ToastMessage {
  id: string;
  type: ToastType;
  message: string;
  duration?: number;
}

interface ToastProps {
  toast: ToastMessage;
  onClose: (id: string) => void;
}

const Toast: React.FC<ToastProps> = ({ toast, onClose }) => {
  useEffect(() => {
    const duration = toast.duration || 5000;
    const timer = setTimeout(() => {
      onClose(toast.id);
    }, duration);

    return () => clearTimeout(timer);
  }, [toast, onClose]);

  const getToastStyles = () => {
    switch (toast.type) {
      case ToastType.Success:
        return 'bg-green-500 dark:bg-green-600';
      case ToastType.Error:
        return 'bg-red-500 dark:bg-red-600';
      case ToastType.Warning:
        return 'bg-yellow-500 dark:bg-yellow-600';
      case ToastType.Info:
        return 'bg-blue-500 dark:bg-blue-600';
      default:
        return 'bg-gray-500 dark:bg-gray-600';
    }
  };

  const getIcon = () => {
    switch (toast.type) {
      case ToastType.Success:
        return '✓';
      case ToastType.Error:
        return '✗';
      case ToastType.Warning:
        return '⚠';
      case ToastType.Info:
        return 'ℹ';
      default:
        return '';
    }
  };

  return (
    <div
      className={`${getToastStyles()} text-white px-6 py-4 rounded-lg shadow-lg flex items-center gap-3 min-w-[300px] max-w-[500px] animate-slide-in`}
      role="alert"
      aria-live="assertive"
    >
      <span className="text-2xl" aria-hidden="true">
        {getIcon()}
      </span>
      <span className="flex-1">{toast.message}</span>
      <button
        onClick={() => onClose(toast.id)}
        className="text-white hover:text-gray-200 text-2xl px-2 transition-colors"
        aria-label="通知を閉じる"
      >
        ×
      </button>
    </div>
  );
};

export default Toast;
