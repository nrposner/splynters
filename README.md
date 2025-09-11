# Splynters
## A Python package for efficient compression of sparse bitmaps

Splynters is a Python wrapping over the [splinter-rs](https://github.com/orbitinghail/splinter-rs) library for zero-copy querying of compressed bitmaps. 

We support Python >=3.8 on recent versions of Windows, MacOS, and manylinux.

We also provide benchmarking against the PyRoaring implementation of Roaring BitMaps, which you can view in the python/benchmarking.ipynb file and run for yourself with run_all_benchmarks.py.

This package can be installed from PyPI using 

```bash
pip install splynters
```

## Examples

### Create a Splinter
Splinter bitmaps can be created from Python iterables, as follows:

```python
import splynters
from splynters import Splinter

data = [1, 5, 789423, 23]
s = Splinter.from_list(data)
```

### Add and remove elements
Elements can be added or removed in the same manner as Python sets: note that we provide both a `.remove()` method which throws an error if it does not find such an element, as well as a `.discard()` method which fails silently in such an event.

```python
s.add(6)

s.remove(5)

# will throw an error! uncomment to see:
# > KeyError: 'remove() could not find the key 99 in the splinter. For a fault-tolerant alternative to remove(), consider discard()'
# s.remove(99) 

s.discard(99) # will fail silently
```

### Check for an element
We can check for the presence of an element using the `.contains()` method. This can be used for either single elements or iterables of elements, and will return either a single boolean or array of booleans.

Note that we also provide a `.contains_many_parallel()` method designed to speed up the process of searching for the presence of many elements at once by parallelizing the search. Note that the regular `.contains()` method is fast enough that the parallel version is not likely to be faster unless you are searching for more than 10,000 elements.

```python
assert(s.contains(1))
assert(not s.contains(99))
assert(s.contains([1, 23, 789423]))
```

We can also do the same for single elements using Python's `in` operator:

```python
assert(1 in s)
assert(not 99 in s)
```

### Set operations
Splinters support all set operations, including bitwise operations:

```python
data1 = [1, 5, 6, 789423, 23]
data2 = [1, 5, 789423, 23, 42]

s1 = Splinter.from_list(data1)
s2 = Splinter.from_list(data2)

splinter_intersection = s1 & s2
splinter_union = s1 | s2
splinter_xor = s1 ^ s2
splinter_sub = s1 - s2

assert(splinter_intersection.to_list() == [1, 5, 23, 789423])
assert(splinter_union.to_list() == [1, 5, 6, 23, 42, 789423]) # note that the output order will be sorted low to high! not necessarily the same as the input order
assert(splinter_xor.to_list() == [6, 42]) 
assert(splinter_sub.to_list() == [6])
```

As well as bitwise assignments:

```python
s1 = Splinter.from_list(data1)
s1 &= s2
assert(s1.to_list() == [1, 5, 23, 789423])

s1 = Splinter.from_list(data1)
s1 |= s2
assert(s1.to_list() == [1, 5, 6, 23, 42, 789423])

s1 = Splinter.from_list(data1)
s1 ^= s2
assert(s1.to_list() == [6, 42])

s1 = Splinter.from_list(data1)
s1 -= s2
assert(s1.to_list() == [6])
```

And also explicit set operations:

```python
data1 = [1, 5, 789423, 23, 42]
data2 = [1, 5, 789423, 42]
data3 = [2, 83, 98798]

s1 = Splinter.from_list(data1)
s2 = Splinter.from_list(data2)
s3 = Splinter.from_list(data3)

s1.isdisjoint(s3)
s2.issubset(s1)
s1.issuperset(s2)
```

And it also supports comparison operators for equality, subsets, and proper subsets:

```python
data1 = [1, 5, 6, 789423, 23]
data2 = [1, 5, 789423, 23, 42]
data3 = [1, 5, 789423, 42]

s1 = Splinter.from_list(data1)
s2 = Splinter.from_list(data2)
s3 = Splinter.from_list(data3)

assert(not s1==s2) # s1 and s2 are not equal
assert(s1 != s2)

assert(not s1 <= s2) # s1 is not a subset of s2
assert(not s1 < s2) # as a consequence of the above, s1 is also not a proper subset of s2
assert(s3 <= s2) # but s3 is a subset of s2
assert(s3 < s2) # and would you look at that, it's also a proper subset!

assert(not s1 >= s2) # s2 is not a subset of s1
assert(not s1 > s2) # s2 is not a proper subset of s1
```

### Serialization, Deserialization, and Pickling

A Splinter object can be serialized to bytes using the `.to_bytes()` method, and deserialized using `.from_bytes()`.

```python
data = [1, 5, 789423, 23]
s = Splinter.from_list(data)

b = s.to_bytes()

s_but_fancy_this_time = Splinter.from_bytes(b)
assert(s == s_but_fancy_this_time)
```

In addition, a splinter object's basic data can be displayed simply by printing it (or in a REPL, using its name), and it can be decompressed to show its internal elements using `.to_list()`:

```python
print(s)
# > SplinterWrapper(len = 4, compressed_byte_size = 35)

print(s.to_list())
# > [1, 5, 23, 789423]
```

Splinter also implements __getstate__ and __setstate__, so that Splinters can be serialized and deserialized with Pickle.

```python
import pickle

s = Splinter.from_list([1, 5, 23, 789423])

pickled_s = pickle.dumps(s)
unpickled_s = pickle.loads(pickled_s)
assert(s == unpickled_s)
```
