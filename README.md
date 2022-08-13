# Search Engine Rust Backend

This is my [search engine](https://github.com/bjroden/search-engine) rewritten in rust.

I wanted to write a new web frontend for it, which meant writing a new backend that interfaces with it. While working
with the old program, I noticed that there were a lot of things about the code structure that could have been better.
I tried refactoring it at first, but I opted for a rewrite instead since the different language choice also allows for much
better tokenizer performance than the old python program (my initial runs show around a 15x speed increase).

# Usage

There are three main programs: the tokenizer, the query program, and the server.

### Tokenizer
The tokenizer will take an input directory, an output directory, and a stopword file as inputs. All files in the input will be indexed and a file mapping will be made
in the output directory, which can then be used by the other two programs. The stopwords file is a list of common words that you do not wish to be indexed.
The number of threads to use for tokenizing can also be specified, with your cpu thread count being the default.

### Query
A CLI to get the top file results for each query. Pass it the directory for the file mapping with the -d flag and the string query with the -q flag.
The number of results can also be adjusted with -n.

### Server

A server that outputs a json mapping of the top results for a query, designed for use with [this frontend](https://github.com/bjroden/search-engine-angular-frontend).
The file mapping directory can be specified with the QUERY_FILES_DIR environment variable.

# TODO:

- [X] Add correct format for fixed-length files
- [X] Write query program
- [X] Reintroduce latin-1 encoding in addition to utf-8
- [X] Re-introduce stopword hashtable
- [ ] Optional: Re-introduce CSS token rule (the version from the old program causes rustc to stack overflow - might be related to the curly brace capture group)
    - [ ] This seems like it would require replacing logos. Logos never backtraces, which means that capturing words followed by curly braces would delete most
          word instances, since it attempts to match CSS first, then fails and simply throws the word away. Since there isn't that much CSS in my data set anyway,
          this is an optional task.
- [X] Optional: let hashtable re-index itself. The python program never did this since it has to write its contents to the dict file and the query program
      has to know the table size, but a 4th file with the table size or a line count on the dict file could probably be introduced.