# corpus-count

Small util to count tokens and optionally character ngrams in a whitespace
tokenized corpus.

Outputs frequency-sorted lists of items.

# Usage
```Bash
# read from file, write ngram and token counts to files
$ corpus-count -c /path/to/corpus.txt -n /path/to/ngram_output.txt \
    -w /path/to/token_output.txt

# read from file, don't count ngrams and write token counts to stdout
$ corpus-count -c /path/to/corpus.txt

# read from stdin, don't count ngrams and write token counts to stdout
$ corpus-count < /path/to/corpus.txt

# read from file, write ngram and token counts to files, filter tokens and
# ngrams appearing less than 30 times. ngrams are counted **before** filtering
# tokens.
$ corpus-count -c /path/to/corpus.txt -n /path/to/ngram_output.txt \
    -w /path/to/token_output.txt --token_min 30 --ngram_min 30
    

# read from file, write ngram and token counts to files, filter out tokens and
# ngrams appearing less than 30 times. Count ngrams **after** filtering tokens.
$ corpus-count -c /path/to/corpus.txt -n /path/to/ngram_output.txt \
    -w /path/to/token_output.txt --token_min 30 --ngram_min 30 --filter_first
``` 

Counting ngrams is determined by giving an argument to the `--ngram_count` or
`-n` flag. Without the `--filter_first` flag, the ngram counts are determined
**before** filtering tokens, therefore tokens which appear less than
`--token_min` times can still contribute to the count of an ngram. If this flag
is set, tokens are filtered first and only in-vocabulary tokens influence the
counts of ngrams.

Per default, tokens are bracketed with "<" and ">" before extracting ngrams. 
This does not affect the tokens, only ngrams and can be toggled through the 
`--no_bracket` flag. 

Minimum and maximum ngram length can be set through the respective `--min_n`
and `--max_n` flags.

# Install

Rust is required, most easily installed through https://rustup.rs.

```Bash
cargo install corpus-count 
```