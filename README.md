# Splynters
## A Python package for efficient compression of sparse bitmaps

Splynters is a Python wrapping over the [splinter-rs](https://github.com/orbitinghail/splinter-rs) library for zero-copy querying of compressed bitmaps. 

We support Python >=3.8 on recent versions of Windows, MacOS, and manylinux.

We also provide benchmarking against the PyRoaring implementation of Roaring BitMaps, which you can view in the python/benchmarking.ipynb file and run for yourself with run_all_benchmarks.py.

This package can be installed from PyPI using 
```
pip install splynters
```


