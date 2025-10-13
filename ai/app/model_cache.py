"""Model caching system for the fruit detection service."""

import asyncio
import threading
from concurrent.futures import ThreadPoolExecutor
from typing import Dict, Optional, Callable, Any, TYPE_CHECKING

from app.logging_config import get_logger

if TYPE_CHECKING:
    from app.models import BaseModel

logger = get_logger(__name__)


class ModelCache:
    """Simple model caching system to improve loading performance."""
    
    def __init__(self, max_size: int = 10):
        self.max_size = max_size
        self._cache: Dict[str, Any] = {}  # Store BaseModel instances
        self._loading_locks: Dict[str, threading.Lock] = {}
        self._executor = ThreadPoolExecutor(max_workers=4)
        self._loop = None

    def get_model(self, model_name: str) -> Optional[Any]:
        """Get a model from cache."""
        return self._cache.get(model_name)

    def set_model(self, model_name: str, model: Any):
        """Set a model in cache."""
        # Remove oldest if cache is full
        if len(self._cache) >= self.max_size:
            # Remove the first item (oldest) in the cache
            oldest_key = next(iter(self._cache))
            del self._cache[oldest_key]
            logger.info(f"Evicted model {oldest_key} from cache due to size limit")
        
        self._cache[model_name] = model
        logger.info(f"Model {model_name} added to cache")
    
    def is_loading(self, model_name: str) -> bool:
        """Check if a model is currently being loaded."""
        return model_name in self._loading_locks
    
    def get_or_create_lock(self, model_name: str) -> threading.Lock:
        """Get or create a lock for a model."""
        if model_name not in self._loading_locks:
            self._loading_locks[model_name] = threading.Lock()
        return self._loading_locks[model_name]
    
    def remove_lock(self, model_name: str):
        """Remove a lock for a model."""
        if model_name in self._loading_locks:
            del self._loading_locks[model_name]
    
    def clear_cache(self):
        """Clear all cached models."""
        self._cache.clear()
        logger.info("Model cache cleared")
    
    async def async_load_model(
        self,
        model_name: str,
        model_factory: Callable[[], Any]
    ) -> Any:
        """Asynchronously load a model with caching and deduplication."""
        
        # Check if model is already in cache
        cached_model = self.get_model(model_name)
        if cached_model is not None:
            logger.info(f"Model {model_name} found in cache")
            return cached_model
        
        # Get or create a lock for this model to prevent duplicate loading
        lock = self.get_or_create_lock(model_name)
        
        # Use the lock to ensure only one loading happens at a time
        with lock:
            # Double-check if model was loaded while we waited for the lock
            cached_model = self.get_model(model_name)
            if cached_model is not None:
                logger.info(f"Model {model_name} found in cache after lock acquired")
                self.remove_lock(model_name)  # Remove lock if model is already loaded
                return cached_model
            
            logger.info(f"Loading model {model_name} asynchronously")
            
            # Load the model in a separate thread
            model = await asyncio.get_event_loop().run_in_executor(
                self._executor, 
                model_factory
            )
            
            # Add the loaded model to cache
            self.set_model(model_name, model)
            
            # Remove the lock as loading is complete
            self.remove_lock(model_name)
            
            logger.info(f"Model {model_name} loaded and cached successfully")
            return model


# Global model cache instance
model_cache = ModelCache()