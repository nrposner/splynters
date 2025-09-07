Desiderata:

For a Python implementation of splinter bitmaps, we'll likely want a few things:
 - we want to make it easy to compress and decompress existing bitmaps into this form. Requires a thoughtful API and some care with I/O operations.
 - We want to convert existing data into a splinter bitmap and back: splinter bitmaps, like roaring, are restricted to u32, so we'll need to perform some mapping of non-u32 data. Carl said that data types like i32 (python default integer) are easy to map and unmap directly losslessly. The real concern is with using growable data types like Strings (not that Python integers aren't growable, smh)
 - for those other data types, we may consider excluding them directly. This is one place where setting up Polars interface will make things easier: we can just query polars for the data type and convert or reject as necessary. 
 - Presumably, people aren't converting the contents of multiple Pandas/Polars columns to bitmaps all at once, are they?
 - we want to provide strong API support for fast set operations: is it possible to do some operator overloading? Quite possibly, if we export the bitmap type as a python class we could define some overloads to sugar over API calls: `res = 12 == bitmap` should return a boolean array (itself a bitmap??) checking which 
 - check if Carl implemented this already, or else get on some bit-twiddling hacks to make operations like NOT, OR, XOR very fast. 
 - Comparing two bitmaps, but also getting values 


We also want to leverage the zero-copy design of the underlying crate: minimal I/O and memory overhead

the encode_to_splinter_ref method in particular should be front and center, maybe available from a Python freeze? Freeze the data once you're done querying it (or if you're going to spend a while querying it before changing it, at least). 
Make this the easy, available behavior, since it's so much more efficient
But in such a way that the user knows they'll want to use this for read-only, but perform the comparatively expensive deserialization process to 
maybe a runtime exception?

how efficient is the insert operation? Can we insert many at once? At that point, just use the FromIterator implementation
note that insert returns a bool value, do we want to do anything with that??
likewise for remove



provide .size from .encoded_size() method
provide .shape from .cardinality() method
is the shape always expected to be unidimensional?

Do we perhaps want to provide support for bundling together multiple bitmaps, for easy conversion into, say, a pandas/polars dataframes of sparse rows?
How would we do that? Do we want to keep the separate 'columns' close together in virtual memory, or is that overkill??





the .encode() method is &self, not &mut self, and no return type? What does it do?

Merge to zip two bitmaps together? good to have in the api

Optimize: check what this actually does and when it should be used: maybe make this available from the api, but certainly make it part of the behavior for higher-level API functions. 
Benchmark Optimize to see what it actually does, and how wasteful it is to call on a bitmap that has already been optimized. If it's more than 1/10th as expensive as calling it on an unoptimized bitmap, prevent this from being called on an optimized one, maybe make use of a PyClass field to check whether it has been modified since last optimization

.contains() is basic, sugar over this with python's `in`?

can the same value be found multiple times??


what is our intended behavior if called on data types we don't want to process, like strings??
Actually, maybe we do want to support strings??
anyway, there will be some data we don't want to support. For these, a runtime exception will be necessary, but also some further errors? If the operation would produce some output, we get the runtime error, but None still gets passed into that output, with the likelihood of cascading errors down the line. Perhaps we can make such operations terminate early and output some safer type? Perhaps even something that preserves progress so an error like this doesn't kill a long-running process? 


How useful is rank()? Is this primarily for under-the-hood operations with sorting, or something end users will want in the API? Ask Carl for his opinion

.select() is very clearly useful, add that in, sugar it from [x] syntax? likewise for .last() on [-1], though supporting general negative iteration is trickier, never done that before. Would have to be some kind of `splinter.select(splinter.cardinality() - x)` operation no? Very slightly less efficient, but how much so? Benchmark how expensive .cardinality is and whether it's > O(1). If we're lucky, Carl thought about this and caches the cardinality already. 
Actually, he must: no way he's traversing each time, it's gotta be a stored length like a vector.


Note also that a Splinter is both Send and Sync! Check to see whether a SplinterRef is as well. 
What can we do with this? Parallelization, obviously, but what kind?? What useful operations can we make available in a high-level API with parallelism in this kind of scenario?

Could we potentially insert or remove multiple values simultaneously?? To create a splinter from many values, it of course makes the most sense to use a FromIterator, but what about after it's created?
For multiple inserts, it may make the most sense to make a separate Splinter out of those first, and then merge them? Does Splinter currently make any use of Rayon or similar??

Ask Carl, but do some benchmarking first.



Also, what about set operations? We'd want to not just do storage and querying for values, but do efficient set operations as well! That's part of what makes Roaring appealing, and not having that functionality would make this a lot less useful. 

I foresee the need for bit-twiddling hacks.


Under what circumstances do we want to use CowSplinter??






Try to get this done and at least a bit polished before the 24th it'll give me some really neat examples and some more insight for the presentation

Contact Hardik about whether this is potentially useful for anything he's doing 


maybe provide a set of force_insert, force_remove etc API functions, for when the end user really does want to add something, even if the bitmap might be frozen right now, and is fine with unfreezing it, adding the element, and freezing it again.

Should optimization happen as a result of the freezing process automatically? Maybe, let's check the API and see if Carl already does this.



