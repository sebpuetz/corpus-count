use std::cmp;
use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{BufRead, BufWriter, Write};

use clap::{App, AppSettings, Arg, ArgMatches};
use stdinout::{Input, Output};

static DEFAULT_CLAP_SETTINGS: &[AppSettings] = &[
    AppSettings::DontCollapseArgsInUsage,
    AppSettings::UnifiedHelpMessage,
];

static CORPUS: &str = "CORPUS";
static FILTER_FIRST: &str = "FILTER_FIRST";
static NO_BRACKET: &str = "NO_BRACKET";
static MAX_N: &str = "MAX_N";
static MIN_N: &str = "MIN_N";
static NGRAM_MIN: &str = "NGRAM_MIN";
static NGRAM_COUNTS: &str = "NGRAM_COUNTS";
static TOKEN_MIN: &str = "TOKEN_MIN";
static TOKEN_COUNTS: &str = "TOKEN_COUNTS";

fn main() {
    let matches = parse_args();
    let corpus = Input::from(matches.value_of(CORPUS));
    let output = Output::from(matches.value_of(TOKEN_COUNTS));
    let mut output = output
        .write()
        .expect("Can't open output to write token counts.");
    let reader = corpus.buf_read().expect("Can't open corpus for reading");
    let ngram_writer = matches.value_of(NGRAM_COUNTS).map(|s| {
        let f = File::create(s).expect("Can't create file to write ngram counts.");
        BufWriter::new(f)
    });
    let bracket = !matches.is_present(NO_BRACKET);
    let filter_first = matches.is_present(FILTER_FIRST);
    let token_min = matches
        .value_of(TOKEN_MIN)
        .map(|v| v.parse::<usize>().expect("Can't parse token min"))
        .unwrap();
    let ngram_min = matches
        .value_of(NGRAM_MIN)
        .map(|v| v.parse::<usize>().expect("Can't parse ngram min"))
        .unwrap();
    let min_n = matches
        .value_of(MIN_N)
        .map(|v| v.parse::<usize>().expect("Can't parse min_n"))
        .unwrap();
    let max_n = matches
        .value_of(MAX_N)
        .map(|v| v.parse::<usize>().expect("Can't parse max_n"))
        .unwrap();
    assert_ne!(min_n, 0, "The minimum n-gram length cannot be zero.");
    assert!(
        min_n <= max_n,
        "The maximum length should be equal to or greater than the minimum length."
    );

    let mut token_counts = HashMap::new();
    for line in reader.lines() {
        let line = line.expect("Can't read line");
        for part in line.split_whitespace() {
            if let Some(cnt) = token_counts.get_mut(part) {
                *cnt += 1;
            } else {
                token_counts.insert(part.to_string(), 1);
            }
        }
    }

    let token_counts = if filter_first {
        counted_into_sorted(token_counts, Some(token_min))
    } else {
        counted_into_sorted(token_counts, None)
    };

    if let Some(mut ngram_writer) = ngram_writer {
        let mut ngram_counts = HashMap::new();
        for (token, count) in token_counts {
            if filter_first && count < token_min {
                continue;
            }
            let token = if bracket {
                let mut b_token = String::with_capacity(token.len() + 2);
                b_token.push('<');
                b_token.push_str(&token);
                b_token.push('>');
                b_token
            } else {
                token
            };
            for ngram in NGrams::new(&token, min_n, max_n) {
                if let Some(idx) = ngram_counts.get_mut(&*ngram) {
                    *idx += count;
                } else {
                    ngram_counts.insert(ngram.to_string(), count);
                }
            }
            writeln!(output, "{}\t{}", token, count).expect("Can't write token counts.");
        }
        counted_into_sorted(ngram_counts, Some(ngram_min))
            .into_iter()
            .for_each(|(ngram, count)| {
                writeln!(ngram_writer, "{}\t{}", ngram, count).expect("Can't write ngram counts.");
            });
    } else {
        token_counts.into_iter().for_each(|(token, count)| {
            writeln!(output, "{}\t{}", token, count).expect("Can't write token counts.");
        });
    }
}

fn counted_into_sorted(
    iter: impl IntoIterator<Item = (String, usize)>,
    filter: Option<usize>,
) -> Vec<(String, usize)> {
    let mut items: Vec<_> = if let Some(min_freq) = filter {
        iter.into_iter()
            .filter(|(_, cnt)| *cnt >= min_freq)
            .collect()
    } else {
        iter.into_iter().collect()
    };
    items.sort_unstable_by(|(t1, c1), (t2, c2)| match c2.cmp(c1) {
        Ordering::Equal => t1.cmp(t2),
        o => o,
    });
    items
}

fn parse_args() -> ArgMatches<'static> {
    App::new("corpus-count")
        .author("Sebastian Pütz")
        .version("0.1.1")
        .settings(DEFAULT_CLAP_SETTINGS)
        .arg(
            Arg::with_name(CORPUS)
                .help("Corpus file")
                .long("corpus")
                .short("c")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(TOKEN_COUNTS)
                .long("token_counts")
                .short("t")
                .help("Token count file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(NGRAM_COUNTS)
                .long("ngram_counts")
                .short("n")
                .help("File for ngram counts")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(TOKEN_MIN)
                .long("token_min")
                .default_value("1")
                .help("Word min count"),
        )
        .arg(
            Arg::with_name(NGRAM_MIN)
                .long("ngram_min")
                .default_value("1")
                .help("Ngram min count"),
        )
        .arg(
            Arg::with_name(MIN_N)
                .long(MIN_N)
                .default_value("3")
                .help("Minimal ngram length to be used.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(MAX_N)
                .long(MAX_N)
                .default_value("6")
                .help("Maximum ngram length to be used.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(FILTER_FIRST)
                .long("filter_first")
                .help("Filter tokens before counting ngrams."),
        )
        .arg(
            Arg::with_name(NO_BRACKET)
                .long("no_bracket")
                .takes_value(false),
        )
        .get_matches()
}

/// Taken from finalfrontier::subtokens
pub struct NGrams<'a> {
    max_n: usize,
    min_n: usize,
    string: &'a str,
    char_offsets: VecDeque<usize>,
    ngram_len: usize,
}

impl<'a> NGrams<'a> {
    /// Create a new n-ngram iterator.
    ///
    /// The iterator will create n-ngrams of length *[min_n, max_n]*
    pub fn new(string: &'a str, min_n: usize, max_n: usize) -> Self {
        // Get the byte offsets of the characters in `string`.
        let char_offsets = string
            .char_indices()
            .map(|(idx, _)| idx)
            .collect::<VecDeque<_>>();

        let ngram_len = cmp::min(max_n, char_offsets.len());

        NGrams {
            min_n,
            max_n,
            string,
            char_offsets,
            ngram_len,
        }
    }
}

impl<'a> Iterator for NGrams<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // If the n-grams for the current suffix are exhausted,
        // move to the next suffix.
        if self.ngram_len < self.min_n {
            // Remove first character, to get the next suffix.
            self.char_offsets.pop_front();

            // If the suffix is smaller than the minimal n-gram
            // length, the iterator is exhausted.
            if self.char_offsets.len() < self.min_n {
                return None;
            }

            // Get the maximum n-gram length for this suffix.
            self.ngram_len = cmp::min(self.max_n, self.char_offsets.len());
        }

        let ngram = if self.ngram_len == self.char_offsets.len() {
            &self.string[self.char_offsets[0]..]
        } else {
            &self.string[self.char_offsets[0]..self.char_offsets[self.ngram_len]]
        };

        self.ngram_len -= 1;

        Some(ngram)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let cap_approx = (self.max_n - self.min_n + 1) * self.char_offsets.len();
        (cap_approx, Some(cap_approx))
    }
}
