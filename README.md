# Search Engine Rust Backend

This is my [search engine](https://github.com/bjroden/search-engine) rewritten in rust.

I wanted to write a new web frontend for it, which meant writing a new backend that interfaces with it. While working
with the old program, I noticed that there were a lot of things about the code structure that could have been better.
I tried refactoring it at first, but I opted for a rewrite instead since the different language choice also allows for much
better tokenizer performance than the old python program (my initial runs show around a 15x speed increase).

# TODO:

- [X] Add correct format for fixed-length files
- [X] Write query program
- [X] Reintroduce latin-1 encoding in addition to utf-8
- [ ] Re-introduce CSS token rule (the version from the old program causes rustc to stack overflow - might be related to the curly brace capture group)
- [ ] Optional: let hashtable re-index itself. The python program never did this since it has to write its contents to the dict file and the query program
      has to know the table size, but a 4th file with the table size or a line count on the dict file could probably be introduced.