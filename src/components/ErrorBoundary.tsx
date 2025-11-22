/**
 * エラーバウンダリコンポーネント
 * Error boundary component to catch and handle React errors
 * Requirement 2.3, 2.4: Handle missing metadata and extraction failures gracefully
 */

import { Component, ErrorInfo, ReactNode } from 'react';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
    };
  }

  static getDerivedStateFromError(error: Error): State {
    // Update state so the next render will show the fallback UI
    return {
      hasError: true,
      error,
      errorInfo: null,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Log error details
    console.error('ErrorBoundary caught an error:', error, errorInfo);
    
    // Log to backend for persistent logging
    this.logErrorToBackend(error, errorInfo);

    // Update state with error info
    this.setState({
      error,
      errorInfo,
    });

    // Call custom error handler if provided
    if (this.props.onError) {
      this.props.onError(error, errorInfo);
    }
  }

  logErrorToBackend(error: Error, errorInfo: ErrorInfo) {
    // Log error details to console for now
    // In production, this could send to a logging service
    const errorLog = {
      timestamp: new Date().toISOString(),
      message: error.message,
      stack: error.stack,
      componentStack: errorInfo.componentStack,
    };
    
    console.error('Error Log:', JSON.stringify(errorLog, null, 2));
  }

  handleReset = () => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
    });
  };

  render() {
    if (this.state.hasError) {
      // Custom fallback UI
      if (this.props.fallback) {
        return this.props.fallback;
      }

      // Default fallback UI
      return (
        <div className="min-h-screen flex items-center justify-center bg-light-bg dark:bg-dark-bg p-4">
          <div className="max-w-2xl w-full bg-light-surface dark:bg-dark-surface rounded-lg shadow-lg p-8">
            <div className="flex items-center gap-4 mb-6">
              <span className="text-6xl" aria-hidden="true">⚠️</span>
              <div>
                <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">
                  エラーが発生しました
                </h1>
                <p className="text-gray-600 dark:text-gray-400 mt-1">
                  Something went wrong
                </p>
              </div>
            </div>

            <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 mb-6">
              <h2 className="text-lg font-semibold text-red-800 dark:text-red-300 mb-2">
                エラー詳細
              </h2>
              <p className="text-red-700 dark:text-red-400 font-mono text-sm break-all">
                {this.state.error?.message}
              </p>
            </div>

            {this.state.error?.stack && (
              <details className="mb-6">
                <summary className="cursor-pointer text-gray-700 dark:text-gray-300 font-semibold mb-2">
                  スタックトレース
                </summary>
                <pre className="bg-gray-100 dark:bg-gray-800 p-4 rounded-lg overflow-auto text-xs text-gray-800 dark:text-gray-200">
                  {this.state.error.stack}
                </pre>
              </details>
            )}

            <div className="flex gap-4">
              <button
                onClick={this.handleReset}
                className="btn btn-primary"
              >
                再試行
              </button>
              <button
                onClick={() => window.location.reload()}
                className="btn bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-900 dark:text-gray-100"
              >
                ページを再読み込み
              </button>
            </div>

            <div className="mt-6 text-sm text-gray-600 dark:text-gray-400">
              <p>
                この問題が続く場合は、アプリケーションを再起動してください。
              </p>
              <p className="mt-2">
                If this problem persists, please restart the application.
              </p>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;
