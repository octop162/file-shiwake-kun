/**
 * エラーリカバリーユーティリティ
 * Error recovery utility for graceful error handling
 * Requirement 2.3, 2.4: Handle missing metadata and extraction failures gracefully
 * Requirement 4.4: Don't delete original file on operation failure
 * Requirement 10.2: Report error and skip file on directory creation failure
 */

import { logger } from './logger';

/**
 * Retry a function with exponential backoff
 */
export async function retryWithBackoff<T>(
  fn: () => Promise<T>,
  maxRetries: number = 3,
  initialDelay: number = 1000,
  context?: string
): Promise<T> {
  let lastError: Error | null = null;
  
  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      const result = await fn();
      if (attempt > 0) {
        logger.info(`Retry successful on attempt ${attempt + 1}`, { context });
      }
      return result;
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      
      if (attempt < maxRetries - 1) {
        const delay = initialDelay * Math.pow(2, attempt);
        logger.warn(
          `Attempt ${attempt + 1} failed, retrying in ${delay}ms`,
          { context, error: lastError.message }
        );
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
  }
  
  logger.error(`All ${maxRetries} retry attempts failed`, {
    context,
    error: lastError?.message
  });
  throw lastError;
}

/**
 * Safely execute a function and return a default value on error
 */
export async function safeExecute<T>(
  fn: () => Promise<T>,
  defaultValue: T,
  context?: string
): Promise<T> {
  try {
    return await fn();
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    logger.warn(`Safe execution failed, returning default value`, {
      context,
      error: errorMessage
    });
    return defaultValue;
  }
}

/**
 * Wrap a function with error boundary
 */
export function withErrorBoundary<T extends (...args: any[]) => any>(
  fn: T,
  onError?: (error: Error) => void
): T {
  return ((...args: any[]) => {
    try {
      const result = fn(...args);
      
      // Handle async functions
      if (result instanceof Promise) {
        return result.catch((error: Error) => {
          logger.error('Async function error', {
            functionName: fn.name,
            error: error.message
          });
          if (onError) {
            onError(error);
          }
          throw error;
        });
      }
      
      return result;
    } catch (error) {
      const err = error instanceof Error ? error : new Error(String(error));
      logger.error('Function error', {
        functionName: fn.name,
        error: err.message
      });
      if (onError) {
        onError(err);
      }
      throw err;
    }
  }) as T;
}

/**
 * Parse error message to user-friendly format
 */
export function parseErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    // Check for common error patterns and provide user-friendly messages
    const message = error.message;
    
    if (message.includes('permission denied') || message.includes('EACCES')) {
      return 'ファイルへのアクセス権限がありません';
    }
    
    if (message.includes('no such file') || message.includes('ENOENT')) {
      return 'ファイルが見つかりません';
    }
    
    if (message.includes('disk space') || message.includes('ENOSPC')) {
      return 'ディスク容量が不足しています';
    }
    
    if (message.includes('file in use') || message.includes('EBUSY')) {
      return 'ファイルが使用中です';
    }
    
    if (message.includes('network') || message.includes('ENETUNREACH')) {
      return 'ネットワークエラーが発生しました';
    }
    
    if (message.includes('timeout')) {
      return '処理がタイムアウトしました';
    }
    
    return message;
  }
  
  return String(error);
}

/**
 * Check if error is recoverable
 */
export function isRecoverableError(error: unknown): boolean {
  if (!(error instanceof Error)) {
    return false;
  }
  
  const message = error.message.toLowerCase();
  
  // Network errors are often recoverable
  if (message.includes('network') || message.includes('timeout')) {
    return true;
  }
  
  // Temporary file locks might be recoverable
  if (message.includes('ebusy') || message.includes('file in use')) {
    return true;
  }
  
  // Permission errors are not recoverable without user action
  if (message.includes('permission') || message.includes('eacces')) {
    return false;
  }
  
  // File not found is not recoverable
  if (message.includes('enoent') || message.includes('no such file')) {
    return false;
  }
  
  // Disk space errors are not recoverable
  if (message.includes('enospc') || message.includes('disk space')) {
    return false;
  }
  
  // Default to not recoverable
  return false;
}

/**
 * Create a recovery strategy for an error
 */
export interface RecoveryStrategy {
  canRecover: boolean;
  suggestedAction: string;
  retryable: boolean;
}

export function getRecoveryStrategy(error: unknown): RecoveryStrategy {
  const message = parseErrorMessage(error);
  const recoverable = isRecoverableError(error);
  
  if (error instanceof Error) {
    const errorMsg = error.message.toLowerCase();
    
    if (errorMsg.includes('permission') || errorMsg.includes('eacces')) {
      return {
        canRecover: false,
        suggestedAction: 'ファイルのアクセス権限を確認してください',
        retryable: false,
      };
    }
    
    if (errorMsg.includes('enospc') || errorMsg.includes('disk space')) {
      return {
        canRecover: false,
        suggestedAction: 'ディスク容量を確保してください',
        retryable: false,
      };
    }
    
    if (errorMsg.includes('ebusy') || errorMsg.includes('file in use')) {
      return {
        canRecover: true,
        suggestedAction: 'ファイルを閉じてから再試行してください',
        retryable: true,
      };
    }
    
    if (errorMsg.includes('network') || errorMsg.includes('timeout')) {
      return {
        canRecover: true,
        suggestedAction: 'ネットワーク接続を確認して再試行してください',
        retryable: true,
      };
    }
  }
  
  return {
    canRecover: recoverable,
    suggestedAction: message,
    retryable: recoverable,
  };
}
