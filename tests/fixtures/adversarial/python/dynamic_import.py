# tests/fixtures/adversarial/python/dynamic_import.py

import importlib
import os

# ⚠️ This looks dead but is imported dynamically
def dynamic_function_a():
    return "Function A"

# ⚠️ This looks dead but is imported dynamically
def dynamic_function_b():
    return "Function B"

# ⚠️ This looks dead but is imported dynamically
def dynamic_function_c():
    return "Function C"

# ⚠️ This looks dead but is dynamically loaded via importlib
class DynamicLoader:
    def __init__(self, module_name):
        self.module_name = module_name
        self._module = None

    def load(self):
        if self._module is None:
            self._module = importlib.import_module(self.module_name)
        return self._module

    def get_function(self, func_name):
        module = self.load()
        return getattr(module, func_name, None)

# ⚠️ This looks dead but is used via dynamic import
def setup_dynamic_imports():
    loader = DynamicLoader(__name__)
    func = loader.get_function("dynamic_function_a")
    if func:
        return func()
    return "Not found"

# Entry point that uses dynamic imports
def main():
    result = setup_dynamic_imports()
    print(result)

    # Also try dynamic imports from other modules
    try:
        module = importlib.import_module("os.path")
        print(module.join("a", "b"))
    except ImportError:
        pass

if __name__ == "__main__":
    main()
