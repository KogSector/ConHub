import logging
import json
import time
import os
import sys
from pathlib import Path
from typing import Dict, Any, Optional
from functools import wraps
from datetime import datetime
import traceback
import psutil
import threading

# Create logs directory if it doesn't exist
LOGS_DIR = Path("logs")
LOGS_DIR.mkdir(exist_ok=True)

class ConHubFormatter(logging.Formatter):
    """Custom formatter for ConHub Haystack service"""
    
    def __init__(self, json_format: bool = False):
        self.json_format = json_format
        if json_format:
            super().__init__()
        else:
            super().__init__(
                fmt='%(asctime)s.%(msecs)03d [%(levelname)s] [%(name)s] %(message)s',
                datefmt='%Y-%m-%d %H:%M:%S'
            )
    
    def format(self, record: logging.LogRecord) -> str:
        if self.json_format:
            log_entry = {
                'timestamp': datetime.fromtimestamp(record.created).isoformat(),
                'level': record.levelname,
                'logger': record.name,
                'message': record.getMessage(),
                'service': 'conhub-haystack-service',
                'version': '1.0.0',
                'environment': os.getenv('ENVIRONMENT', 'development'),
                'hostname': os.uname().nodename,
                'pid': os.getpid(),
                'thread': threading.current_thread().name
            }
            
            # Add extra fields from LogRecord
            for key, value in record.__dict__.items():
                if key not in ['name', 'msg', 'args', 'pathname', 'lineno', 'funcName', 
                              'created', 'msecs', 'relativeCreated', 'thread', 'threadName',
                              'processName', 'process', 'module', 'filename', 'levelno',
                              'levelname', 'getMessage', 'exc_info', 'exc_text', 'stack_info']:
                    log_entry[key] = value
            
            # Add exception info if present
            if record.exc_info:
                log_entry['exception'] = {
                    'type': record.exc_info[0].__name__ if record.exc_info[0] else None,
                    'message': str(record.exc_info[1]) if record.exc_info[1] else None,
                    'traceback': traceback.format_exception(*record.exc_info) if record.exc_info else None
                }
            
            return json.dumps(log_entry)
        else:
            return super().format(record)

def setup_logging():
    """Setup comprehensive logging for Haystack service"""
    
    # Determine if we're in production
    is_production = os.getenv('ENVIRONMENT') == 'production'
    log_level = os.getenv('LOG_LEVEL', 'INFO' if is_production else 'DEBUG')
    
    # Create root logger
    root_logger = logging.getLogger()
    root_logger.setLevel(getattr(logging, log_level.upper()))
    
    # Clear existing handlers
    root_logger.handlers.clear()
    
    # Console handler for development
    if not is_production:
        console_handler = logging.StreamHandler(sys.stdout)
        console_handler.setLevel(logging.DEBUG)
        console_handler.setFormatter(ConHubFormatter(json_format=False))
        root_logger.addHandler(console_handler)
    
    # File handlers
    handlers = [
        {
            'filename': LOGS_DIR / 'haystack-combined.log',
            'level': logging.DEBUG,
            'max_bytes': 10 * 1024 * 1024,  # 10MB
            'backup_count': 5
        },
        {
            'filename': LOGS_DIR / 'haystack-errors.log',
            'level': logging.ERROR,
            'max_bytes': 10 * 1024 * 1024,  # 10MB
            'backup_count': 5
        },
        {
            'filename': LOGS_DIR / 'haystack-performance.log',
            'level': logging.INFO,
            'max_bytes': 10 * 1024 * 1024,  # 10MB
            'backup_count': 5,
            'filter_category': 'performance'
        },
        {
            'filename': LOGS_DIR / 'haystack-documents.log',
            'level': logging.INFO,
            'max_bytes': 10 * 1024 * 1024,  # 10MB
            'backup_count': 5,
            'filter_category': 'document'
        }
    ]
    
    for handler_config in handlers:
        from logging.handlers import RotatingFileHandler
        
        handler = RotatingFileHandler(
            handler_config['filename'],
            maxBytes=handler_config['max_bytes'],
            backupCount=handler_config['backup_count']
        )
        handler.setLevel(handler_config['level'])
        handler.setFormatter(ConHubFormatter(json_format=True))
        
        # Add filter if specified
        if 'filter_category' in handler_config:
            handler.addFilter(lambda record, cat=handler_config['filter_category']: 
                            getattr(record, 'category', None) == cat)
        
        root_logger.addHandler(handler)
    
    # Setup uvicorn logging
    uvicorn_logger = logging.getLogger("uvicorn")
    uvicorn_logger.setLevel(logging.INFO)
    
    uvicorn_access_logger = logging.getLogger("uvicorn.access")
    uvicorn_access_logger.setLevel(logging.INFO)
    
    # Log startup info
    logger = logging.getLogger(__name__)
    logger.info("Haystack service logging initialized", extra={
        'category': 'startup',
        'log_level': log_level,
        'is_production': is_production,
        'python_version': sys.version,
        'pid': os.getpid()
    })

class PerformanceMonitor:
    """Performance monitoring and logging for Haystack service"""
    
    def __init__(self):
        self.logger = logging.getLogger(f"{__name__}.performance")
        self.active_operations: Dict[str, float] = {}
    
    def start_operation(self, operation_id: str, operation_type: str, context: Optional[Dict[str, Any]] = None):
        """Start timing an operation"""
        start_time = time.time()
        self.active_operations[operation_id] = start_time
        
        self.logger.debug("Operation started", extra={
            'category': 'performance',
            'operation_id': operation_id,
            'operation_type': operation_type,
            'start_time': start_time,
            **(context or {})
        })
    
    def end_operation(self, operation_id: str, operation_type: str, 
                     success: bool = True, context: Optional[Dict[str, Any]] = None):
        """End timing an operation"""
        end_time = time.time()
        start_time = self.active_operations.pop(operation_id, None)
        
        if start_time is None:
            self.logger.warning("Operation timing not found", extra={
                'category': 'performance',
                'operation_id': operation_id,
                'operation_type': operation_type
            })
            return 0
        
        duration = end_time - start_time
        
        log_level = logging.INFO
        if duration > 10:  # Slow operation (> 10 seconds)
            log_level = logging.WARNING
        
        self.logger.log(log_level, "Operation completed", extra={
            'category': 'performance',
            'operation_id': operation_id,
            'operation_type': operation_type,
            'duration': round(duration, 3),
            'success': success,
            'is_slow': duration > 10,
            **(context or {})
        })
        
        return duration
    
    def log_metric(self, metric_name: str, value: float, unit: str = '', 
                   context: Optional[Dict[str, Any]] = None):
        """Log a performance metric"""
        self.logger.info("Performance metric", extra={
            'category': 'performance',
            'metric_name': metric_name,
            'value': value,
            'unit': unit,
            **(context or {})
        })
    
    def log_system_metrics(self):
        """Log current system metrics"""
        try:
            process = psutil.Process()
            memory_info = process.memory_info()
            cpu_percent = process.cpu_percent()
            
            self.logger.info("System metrics", extra={
                'category': 'performance',
                'memory_rss_mb': round(memory_info.rss / 1024 / 1024, 2),
                'memory_vms_mb': round(memory_info.vms / 1024 / 1024, 2),
                'cpu_percent': cpu_percent,
                'num_threads': process.num_threads(),
                'num_fds': process.num_fds() if hasattr(process, 'num_fds') else None
            })
        except Exception as e:
            self.logger.error("Failed to log system metrics", extra={
                'category': 'performance',
                'error': str(e)
            })

class DocumentOperationLogger:
    """Logger for document processing operations"""
    
    def __init__(self):
        self.logger = logging.getLogger(f"{__name__}.document")
    
    def log_indexing_start(self, document_count: int, source: str, context: Optional[Dict[str, Any]] = None):
        """Log start of document indexing"""
        self.logger.info("Document indexing started", extra={
            'category': 'document',
            'operation': 'indexing_start',
            'document_count': document_count,
            'source': source,
            **(context or {})
        })
    
    def log_indexing_complete(self, document_count: int, success_count: int, 
                            error_count: int, duration: float, source: str):
        """Log completion of document indexing"""
        self.logger.info("Document indexing completed", extra={
            'category': 'document',
            'operation': 'indexing_complete',
            'document_count': document_count,
            'success_count': success_count,
            'error_count': error_count,
            'duration': round(duration, 3),
            'source': source,
            'success_rate': round((success_count / document_count) * 100, 2) if document_count > 0 else 0
        })
    
    def log_document_processed(self, document_id: str, document_type: str, 
                             success: bool, processing_time: float, 
                             error: Optional[str] = None):
        """Log individual document processing"""
        log_level = logging.INFO if success else logging.ERROR
        
        self.logger.log(log_level, "Document processed", extra={
            'category': 'document',
            'operation': 'document_process',
            'document_id': document_id,
            'document_type': document_type,
            'success': success,
            'processing_time': round(processing_time, 3),
            'error': error
        })
    
    def log_search_operation(self, query: str, result_count: int, 
                           duration: float, filters: Optional[Dict[str, Any]] = None):
        """Log search operation"""
        self.logger.info("Search performed", extra={
            'category': 'document',
            'operation': 'search',
            'query_length': len(query),
            'query_preview': query[:50] + ('...' if len(query) > 50 else ''),
            'result_count': result_count,
            'duration': round(duration, 3),
            'filters': filters or {}
        })
    
    def log_qa_operation(self, question: str, answer_length: int, 
                        confidence: Optional[float], duration: float,
                        context_count: int):
        """Log Q&A operation"""
        self.logger.info("Q&A performed", extra={
            'category': 'document',
            'operation': 'qa',
            'question_length': len(question),
            'question_preview': question[:50] + ('...' if len(question) > 50 else ''),
            'answer_length': answer_length,
            'confidence': confidence,
            'duration': round(duration, 3),
            'context_count': context_count
        })

def timed_operation(operation_type: str, logger_instance: Optional[logging.Logger] = None):
    """Decorator for timing operations"""
    def decorator(func):
        @wraps(func)
        def wrapper(*args, **kwargs):
            operation_id = f"{operation_type}_{int(time.time() * 1000)}"
            start_time = time.time()
            
            logger = logger_instance or logging.getLogger(func.__module__)
            logger.debug(f"Starting {operation_type}", extra={
                'category': 'performance',
                'operation_id': operation_id,
                'operation_type': operation_type,
                'function': func.__name__
            })
            
            try:
                result = func(*args, **kwargs)
                duration = time.time() - start_time
                
                logger.info(f"Completed {operation_type}", extra={
                    'category': 'performance',
                    'operation_id': operation_id,
                    'operation_type': operation_type,
                    'function': func.__name__,
                    'duration': round(duration, 3),
                    'success': True
                })
                
                return result
            except Exception as e:
                duration = time.time() - start_time
                
                logger.error(f"Failed {operation_type}", extra={
                    'category': 'performance',
                    'operation_id': operation_id,
                    'operation_type': operation_type,
                    'function': func.__name__,
                    'duration': round(duration, 3),
                    'success': False,
                    'error': str(e),
                    'exception_type': type(e).__name__
                })
                
                raise
        
        return wrapper
    return decorator

# Create global instances
performance_monitor = PerformanceMonitor()
document_logger = DocumentOperationLogger()

# Setup logging when module is imported
setup_logging()

# Start periodic system metrics logging
def start_system_monitoring():
    """Start periodic system metrics logging"""
    import threading
    import time
    
    def log_metrics():
        while True:
            try:
                performance_monitor.log_system_metrics()
                time.sleep(60)  # Log every minute
            except Exception as e:
                logging.getLogger(__name__).error(f"Error in system monitoring: {e}")
                time.sleep(60)
    
    thread = threading.Thread(target=log_metrics, daemon=True)
    thread.start()
    
    logger = logging.getLogger(__name__)
    logger.info("System monitoring started", extra={'category': 'startup'})

# Auto-start system monitoring
if os.getenv('ENVIRONMENT') != 'test':
    start_system_monitoring()