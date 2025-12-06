import sys
import os
import logging
from typing import List, Dict, Any, Callable
from PySide6.QtWidgets import QApplication, QMainWindow, QMessageBox
from PySide6.QtCore import QThread, Signal, Slot, QObject, QMutex, QWaitCondition
from PySide6.QtGui import QIcon

from ui.main_view import MainView
from ui.preview_window import PreviewWindow
from ui.results_view import ResultsView
from ui.conflict_dialog import ConflictDialog
from logic.file_processor import FileProcessor
from data.config_manager import ConfigManager

# --- Basic Logging Setup ---
def get_base_path():
    """ Get the base path for data files, for PyInstaller, Nuitka, or Dev. """
    # PyInstaller
    if getattr(sys, 'frozen', False):
        return os.path.dirname(sys.executable)
    
    # Nuitka (Compiled)
    # Note: In Nuitka --onefile, __file__ points to temp dir, but sys.argv[0] points to original exe
    if "__compiled__" in globals():
        return os.path.dirname(os.path.abspath(sys.argv[0]))
        
    # Development (Normal Python)
    return os.path.dirname(os.path.abspath(__file__))

# Setup logging to file in the base path
try:
    base_path = get_base_path()
    log_file_path = os.path.join(base_path, 'debug.log')
    
    # Determine log level: INFO for compiled builds, DEBUG for development
    is_compiled = getattr(sys, 'frozen', False) or "__compiled__" in globals()
    log_level = logging.INFO if is_compiled else logging.DEBUG

    logging.basicConfig(
        level=log_level,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        filename=log_file_path,
        filemode='w',
        encoding='utf-8'
    )
    logger = logging.getLogger(__name__)
    
    # Log path details for debugging
    logger.info("=== Application Startup ===")
    logger.info(f"Calculated Base Path: {base_path}")
    logger.info(f"sys.argv[0]: {sys.argv[0]}")
    logger.info(f"sys.executable: {sys.executable}")
    logger.info(f"__file__: {__file__}")
    logger.info(f"Is Frozen (PyInstaller): {getattr(sys, 'frozen', False)}")
    logger.info(f"Is Compiled (Nuitka): {'__compiled__' in globals()}")
    
except Exception as e:
    # Fallback if logging setup fails (e.g. permission denied in base path)
    print(f"Failed to setup logging: {e}")
    # Still try to continue, maybe with a default logger to stderr
    logging.basicConfig(level=logging.DEBUG)
    logger = logging.getLogger(__name__)
    logger.error(f"Logging setup failed: {e}")

# Redirect stdout/stderr to logger for catching crashes
class StreamToLogger(object):
    def __init__(self, logger, log_level=logging.INFO):
        self.logger = logger
        self.log_level = log_level
        self.linebuf = ''

    def write(self, buf):
        for line in buf.rstrip().splitlines():
            self.logger.log(self.log_level, line.rstrip())
    
    def flush(self):
        pass

sys.stdout = StreamToLogger(logger, logging.INFO)
sys.stderr = StreamToLogger(logger, logging.ERROR)


class WorkerSignals(QObject):
    progress = Signal(dict) # {current, total, phase}
    result = Signal(dict)   # {type: 'discovery'|'execution', data: ...}
    conflict = Signal(str, str) # source, dest
    error = Signal(str)
    finished = Signal()
    cancelled = Signal()

class Worker(QThread):
    """
    Worker thread for running file operations.
    """
    def __init__(self, task_type: str, file_processor: FileProcessor, **kwargs):
        super().__init__()
        self.task_type = task_type
        self.file_processor = file_processor
        self.kwargs = kwargs
        self.signals = WorkerSignals()
        
        # Thread synchronization for conflict handling
        self.mutex = QMutex()
        self.condition = QWaitCondition()
        self.conflict_resolution = None

    def run(self):
        try:
            if self.task_type == 'discovery':
                self._run_discovery()
            elif self.task_type == 'execution':
                self._run_execution()
        except InterruptedError:
            logger.info("Worker thread interrupted by user.")
            self.signals.cancelled.emit()
        except Exception as e:
            logger.exception("Error in worker thread")
            self.signals.error.emit(str(e))
        finally:
            self.signals.finished.emit()

    def _run_discovery(self):
        file_paths = self.kwargs['file_paths']
        rule_id = self.kwargs['rule_id']
        
        def progress_cb(current, total=0):
            phase = "ファイルリスト作成中" if total == 0 else "ファイル分析中"
            self.signals.progress.emit({"current": current, "total": total, "phase": phase})

        planned_ops = self.file_processor.discover_operations(
            file_paths, rule_id, progress_cb, cancel_check=self.isInterruptionRequested
        )
        
        if self.isInterruptionRequested():
             raise InterruptedError()

        self.signals.result.emit({
            "type": "discovery",
            "results": planned_ops,
            "file_paths": file_paths,
            "rule_id": rule_id
        })

    def _run_execution(self):
        planned_ops = self.kwargs['planned_ops']
        total_ops = len(planned_ops)
        results = []

        for i, plan in enumerate(planned_ops):
            if self.isInterruptionRequested():
                raise InterruptedError()
                
            self.signals.progress.emit({"current": i + 1, "total": total_ops, "phase": "ファイル処理実行中"})
            
            # The file_processor uses the conflict_handler passed in __init__
            # We need to ensure it calls OUR handle_conflict method
            result = self.file_processor.execute_operation(
                file_path=plan['file_path'],
                rule=plan['rule'],
                dest_path=plan['dest_path']
            )
            results.append(result)

        self.signals.result.emit({
            "type": "execution",
            "results": results
        })

    def handle_conflict(self, source_path: str, dest_path: str) -> str:
        """
        Called by FileProcessor (running in this thread) when a conflict occurs.
        """
        logger.debug(f"Conflict detected in worker: {source_path} -> {dest_path}")
        self.signals.conflict.emit(source_path, dest_path)
        
        # Wait for the main thread to resolve the conflict
        self.mutex.lock()
        self.condition.wait(self.mutex)
        resolution = self.conflict_resolution
        self.mutex.unlock()
        
        logger.debug(f"Conflict resolved with: {resolution}")
        return resolution

    def wake_up(self, resolution: dict):
        """
        Called by the main thread to wake up the worker with a resolution.
        """
        self.mutex.lock()
        self.conflict_resolution = resolution
        self.condition.wakeAll()
        self.mutex.unlock()


class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("ファイル仕分君")
        self.resize(900, 750)
        
        # Center window
        if self.screen():
            geometry = self.screen().availableGeometry()
            self.move(
                geometry.x() + (geometry.width() - 900) // 2,
                geometry.y() + (geometry.height() - 750) // 2
            )

        try:
            config_path = os.path.join(get_base_path(), 'config.json')
            self.config_manager = ConfigManager(config_path)
            self.config = self.config_manager.load_config()
        except Exception as e:
            QMessageBox.critical(self, "設定エラー", f"設定ファイルの読み込みに失敗しました。\n{e}")
            sys.exit(1)

        # Force preview mode to always be on
        self.config['preview_mode'] = True

        self.main_view = MainView(
            self, 
            config=self.config,
            on_save=self.save_config,
            on_file_drop=self.handle_file_drop
        )
        self.main_view.cancelRequested.connect(self.cancel_processing)
        self.setCentralWidget(self.main_view)

        # Restore last selection
        last_selected_id = self.config.get("last_selected_rule_id")
        if last_selected_id:
            self.main_view.select_rule_by_id(last_selected_id)
            
        self.worker = None

    def closeEvent(self, event):
        logging.debug("Close event detected, saving last selection.")
        selected_id = self.main_view.get_selected_rule_id()
        self.config["last_selected_rule_id"] = selected_id
        self.save_config(self.config)
        
        # Ensure worker is stopped
        if self.worker and self.worker.isRunning():
            self.worker.requestInterruption()
            self.worker.wait(1000)
            
        event.accept()

    def save_config(self, new_config):
        logging.debug(f"Saving new configuration.")
        self.config = new_config
        self.config_manager.save_config(self.config)

    def handle_file_drop(self, file_paths: list[str]):
        selected_rule_id = self.main_view.get_selected_rule_id()
        if not selected_rule_id:
            QMessageBox.warning(self, "ルール未選択", "処理を実行する前に、リストからルールを1つ選択してください。")
            return
            
        logging.info(f"Files dropped, starting background processing for {len(file_paths)} files.")
        self.main_view.show_processing_state(True)
        self.main_view.update_progress_text("ファイル分析中...")

        self.start_worker('discovery', file_paths=file_paths, rule_id=selected_rule_id)

    def cancel_processing(self):
        if self.worker and self.worker.isRunning():
            logging.info("Cancellation requested by user.")
            self.worker.requestInterruption()
            self.main_view.update_progress_text("キャンセル中...")
            self.main_view.drop_zone.cancel_btn.setEnabled(False)

    def start_worker(self, task_type, **kwargs):
        # Create a NEW FileProcessor for each task to ensure clean state
        # Pass the worker's handle_conflict method later? No, need to pass it in init.
        # But we haven't created the worker yet.
        # Solution: Create worker first, then create processor with worker.handle_conflict
        
        # We need a temporary object or a lambda that delegates to the future worker instance?
        # Or simpler: The worker is the one running the processor. 
        
        # Let's define the processor inside the worker creation or pass a factory.
        # Actually, simpler:
        
        self.worker = Worker(task_type, None, **kwargs) # Pass None first
        
        # Now create the processor with the worker's bound method
        processor = FileProcessor(self.config, conflict_handler=self.worker.handle_conflict)
        self.worker.file_processor = processor
        
        self.worker.signals.progress.connect(self.on_worker_progress)
        self.worker.signals.result.connect(self.on_worker_result)
        self.worker.signals.conflict.connect(self.on_worker_conflict)
        self.worker.signals.error.connect(self.on_worker_error)
        self.worker.signals.cancelled.connect(self.on_worker_cancelled)
        self.worker.start()

    @Slot(dict)
    def on_worker_progress(self, data):
        current = data["current"]
        total = data["total"]
        phase = data.get("phase", "処理中")

        if total > 0:
            progress_text = f"{phase}: {current} / {total} ファイル"
        else:
            progress_text = f"{phase}: {current} ファイル発見"
        self.main_view.update_progress_text(progress_text)

    @Slot(dict)
    def on_worker_result(self, data):
        result_type = data["type"]
        
        if result_type == "discovery":
            self._handle_discovery_result(data)
        elif result_type == "execution":
            self._handle_execution_result(data)
            
    @Slot()
    def on_worker_cancelled(self):
        self.main_view.show_processing_state(False)
        self.main_view.update_progress_text("処理がキャンセルされました")
        self.main_view.drop_zone.cancel_btn.setEnabled(True)
        QMessageBox.information(self, "キャンセル", "処理がユーザーによってキャンセルされました。")

    def _handle_discovery_result(self, data):
        self.main_view.show_processing_state(False)
        planned_ops = data["results"]
        
        executable_plans = [op for op in planned_ops if not op.get('error')]
        if not executable_plans:
            self.main_view.update_progress_text("") 
            QMessageBox.information(self, "プレビュー", "選択されたルールにマッチするファイルはありませんでした。")
            return

        if self.config.get('preview_mode', True):
            preview_dialog = PreviewWindow(self, executable_plans)
            if preview_dialog.exec(): # Accepted
                logging.info("Preview confirmed. Executing operations.")
                self.main_view.show_processing_state(True)
                self.main_view.update_progress_text("ファイル処理実行中...")
                self.start_worker('execution', planned_ops=executable_plans)
            else:
                self.main_view.update_progress_text("")
                logging.info("Preview cancelled by user.")
        else:
            # Direct execution
             self.start_worker('execution', planned_ops=executable_plans)

    def _handle_execution_result(self, data):
        self.main_view.show_processing_state(False)
        self.main_view.update_progress_text("")
        results = data["results"]
        
        dialog = ResultsView(self, results)
        dialog.exec()

    @Slot(str, str)
    def on_worker_conflict(self, source, dest):
        """
        Show conflict dialog in the main thread.
        """
        dialog = ConflictDialog(self, source, dest)
        if dialog.exec():
            resolution = dialog.result
        else:
            # If dialog cancelled (e.g. closed window), default to skip?
            resolution = {"resolution": "skip", "apply_to_all": False}
            
        # Pass result back to worker
        if self.worker:
            self.worker.wake_up(resolution)

    @Slot(str)
    def on_worker_error(self, message):
        self.main_view.show_processing_state(False)
        self.main_view.update_progress_text("エラー発生")
        QMessageBox.critical(self, "エラー", f"処理中にエラーが発生しました:\n{message}")

# Global exception hook to show errors in GUI
def excepthook(exc_type, exc_value, exc_traceback):
    logger.error("Uncaught exception", exc_info=(exc_type, exc_value, exc_traceback))
    msg = f"{exc_type.__name__}: {exc_value}"
    # Ensure we have a QApplication instance before showing message box
    if QApplication.instance():
        QMessageBox.critical(None, "予期せぬエラー", f"アプリケーションでエラーが発生しました:\n{msg}\n\n詳細はログファイルを確認してください。")
    else:
        # Fallback if GUI not ready
        sys.__excepthook__(exc_type, exc_value, exc_traceback)

sys.excepthook = excepthook

if __name__ == "__main__":
    app = QApplication(sys.argv)
    window = MainWindow()
    window.show()
    sys.exit(app.exec())