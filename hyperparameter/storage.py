import threading
from typing import Any, Dict, Iterable


class Storage:
    """Base class for all storage implementations"""

    # storage operations
    def child(self) -> "Storage":
        return None

    def storage(self) -> Dict[str, Any]:
        pass

    # dict operations
    def keys(self) -> Iterable:
        pass

    def update(self, kws: Dict[str, Any]) -> None:
        return None

    def clear(self):
        pass

    def __iter__(self):
        pass

    # kv operations
    def get(self, name: str) -> Any:
        return None

    def put(self, name: str, value: Any) -> None:
        return None

    # context operations
    def enter(self):
        pass

    def exit(self):
        pass

    @staticmethod
    def current():
        pass


class TLSKVStorage(Storage):
    """Pure Python implementation of a key-value storage"""

    __slots__ = ("_storage", "_parent", "_accessor")
    tls = threading.local()

    def __init__(self, parent=None, accessor=None) -> None:
        self._storage = None
        self._parent = parent
        self._accessor = accessor
        super().__init__()

        if hasattr(TLSKVStorage.tls, "his") and len(TLSKVStorage.tls.his) > 0:
            parent = TLSKVStorage.tls.his[-1]
            self.update(parent._storage)

    def __iter__(self):
        return iter(self._storage.items())

    def child(self) -> "Storage":
        obj = TLSKVStorage(self, self._accessor)
        obj.update(self._storage)
        return obj

    def storage(self) -> Dict[str, Any]:
        return self._storage

    def keys(self) -> Iterable:
        return self._storage.keys()

    def update(self, kws: Dict[str, Any]) -> None:
        if self._storage is None:
            self._storage = {}

        storage = self._storage

        def _update(values={}, prefix=None):
            for k, v in values.items():
                key = f"{prefix}.{k}" if prefix is not None else f"{k}"
                if isinstance(v, dict):
                    _update(v, prefix=key)
                else:
                    storage[key] = v

        return _update(kws, prefix=None)

    def clear(self):
        self._storage.clear()

    def get(self, name: str) -> Any:
        if name in self.__slots__:
            return self.__dict__[name]
        curr = self
        while curr is not None:
            if name in curr._storage:
                return curr._storage[name]
            curr = curr._parent
        return self._accessor(self, name)

    def put(self, name: str, value: Any) -> None:
        if name in self.__slots__:
            return self.__dict__.__setitem__(name, value)
        return self.update({name: value})

    def enter(self):
        if not hasattr(TLSKVStorage.tls, "his"):
            TLSKVStorage.tls.his = []
        TLSKVStorage.tls.his.append(self)
        return TLSKVStorage.tls.his[-1]

    def exit(self):
        TLSKVStorage.tls.his.pop()

    @staticmethod
    def current():
        if not hasattr(TLSKVStorage.tls, "his") or len(TLSKVStorage.tls.his) == 0:
            TLSKVStorage.tls.his = [TLSKVStorage()]
        return TLSKVStorage.tls.his[-1]
