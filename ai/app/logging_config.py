"""Structured logging configuration for the fruit detection service."""

import logging
import sys
from typing import Optional

from app.config import config


def setup_logging(
    name: str = __name__,
    level: Optional[int] = None,
    log_format: Optional[str] = None,
    log_file: Optional[str] = None
) -> logging.Logger:
    """
    Set up structured logging for the service.
    
    Args:
        name: Logger name
        level: Logging level (defaults to INFO or DEBUG based on environment)
        log_format: Custom log format
        log_file: Optional file to log to
    
    Returns:
        Configured logger instance
    """
    # Use provided level or default based on environment
    if level is None:
        level = logging.DEBUG if config.DEBUG else logging.INFO
    
    # Create logger
    logger = logging.getLogger(name)
    logger.setLevel(level)
    
    # Avoid adding multiple handlers if logger already has them
    if logger.handlers:
        return logger
    
    # Create console handler
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setLevel(level)
    
    # Create formatter
    if log_format is None:
        log_format = (
            "%(asctime)s - %(name)s - %(levelname)s - "
            "[%(filename)s:%(lineno)d] - %(funcName)s() - %(message)s"
        )
    
    formatter = logging.Formatter(log_format)
    console_handler.setFormatter(formatter)
    
    # Add console handler to logger
    logger.addHandler(console_handler)
    
    # Optionally add file handler
    if log_file:
        file_handler = logging.FileHandler(log_file)
        file_handler.setLevel(level)
        file_handler.setFormatter(formatter)
        logger.addHandler(file_handler)
    
    # Prevent propagation to root logger to avoid duplicate logs
    logger.propagate = False
    
    return logger


# Global logger setup
def get_logger(name: str = __name__) -> logging.Logger:
    """Get a configured logger instance."""
    return setup_logging(name=name)


# Create a global logger instance for the app
logger = get_logger("fruit_detection")