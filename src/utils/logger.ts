/**
 * フロントエンドロギングユーティリティ
 * Frontend logging utility for detailed log output
 * Requirement 8.1, 8.5: Log all file operations for troubleshooting
 */

export enum LogLevel {
  DEBUG = 'DEBUG',
  INFO = 'INFO',
  WARN = 'WARN',
  ERROR = 'ERROR',
}

interface LogEntry {
  timestamp: string;
  level: LogLevel;
  message: string;
  data?: any;
}

class Logger {
  private logs: LogEntry[] = [];
  private maxLogs: number = 1000;

  /**
   * Log a debug message
   */
  debug(message: string, data?: any) {
    this.log(LogLevel.DEBUG, message, data);
    console.debug(`[DEBUG] ${message}`, data);
  }

  /**
   * Log an info message
   */
  info(message: string, data?: any) {
    this.log(LogLevel.INFO, message, data);
    console.info(`[INFO] ${message}`, data);
  }

  /**
   * Log a warning message
   */
  warn(message: string, data?: any) {
    this.log(LogLevel.WARN, message, data);
    console.warn(`[WARN] ${message}`, data);
  }

  /**
   * Log an error message
   */
  error(message: string, data?: any) {
    this.log(LogLevel.ERROR, message, data);
    console.error(`[ERROR] ${message}`, data);
  }

  /**
   * Internal log method
   */
  private log(level: LogLevel, message: string, data?: any) {
    const entry: LogEntry = {
      timestamp: new Date().toISOString(),
      level,
      message,
      data,
    };

    this.logs.push(entry);

    // Keep only the most recent logs
    if (this.logs.length > this.maxLogs) {
      this.logs.shift();
    }
  }

  /**
   * Get all logs
   */
  getLogs(): LogEntry[] {
    return [...this.logs];
  }

  /**
   * Get logs filtered by level
   */
  getLogsByLevel(level: LogLevel): LogEntry[] {
    return this.logs.filter((log) => log.level === level);
  }

  /**
   * Clear all logs
   */
  clearLogs() {
    this.logs = [];
  }

  /**
   * Export logs as JSON string
   */
  exportLogs(): string {
    return JSON.stringify(this.logs, null, 2);
  }

  /**
   * Download logs as a file
   */
  downloadLogs() {
    const logsJson = this.exportLogs();
    const blob = new Blob([logsJson], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `file-shiwake-kun-logs-${new Date().toISOString()}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }
}

// Export singleton instance
export const logger = new Logger();
