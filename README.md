# Search Engine Rust Backend

This is my [search engine](https://github.com/bjroden/search-engine) rewritten in rust.

I wanted to write a new web frontend for it, which meant writing a new backend that interfaces with it. While working
with the old program, I noticed that there were a lot of things about the code structure that could have been better.
I tried refactoring it at first, but I opted for a rewrite instead since the different language choice also allows for much
better tokenizer performance than the old python program (my initial runs show around a 15x speed increase).