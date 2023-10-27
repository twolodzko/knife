# It's just a sharper knife 🔪

`knife` is like the `cut` command, but delimits fields with whitespaces, any whitespaces.

Did you ever felt frustrated when trying to extract whitespace delimited fields using tools like `cut`?
_Should I split on tabs or spaces? How many?_

```shell
$ echo "Mary   had a     little  lamb." | cut -f 2-4   
Mary   had a     little  lamb.
$ echo "Mary   had a     little  lamb." | cut -d' ' -f 2-4
  had
```

I created `knife` exactly for this purpose. It splits the input strings by whitespaces using Rust's [`std::str::SplitWhitespace`]
and extracts fields specified using a pattern language like the one used by `cut` (e.g. `1,3,5-8` for the fields 1, 3,
and 5 to 8 inclusively).

```shell
$ echo "Mary   had a     little  lamb." | knife 2-4   
had a little
```

That's it, it doesn't do anything more. If you need more customizable search or more bells and whistles, use dedicated
tools like `grep`, `sed`, `awk`, etc or the `cut` itself.

## Installation

To install it run:

```shell
cargo install --git https://github.com/twolodzko/knife.git
```

## Benchmarks

`knife` runs in linear time. It is faster than `cut` in some common scenarios and slower or equal in others.
It should perform roughly the same or better than `cut`.

```shell
$ hyperfine -N -r 10000 'echo "Marry had a little lamb." | cut -d" " -f 2-4' 'echo "Marry had a little lamb." | knife 2-4'
Benchmark 1: echo "Marry had a little lamb." | cut -d" " -f 2-4
  Time (mean ± σ):       1.7 ms ±   0.4 ms    [User: 1.0 ms, System: 0.6 ms]
  Range (min … max):     1.0 ms …   4.9 ms    10000 runs
 
Benchmark 2: echo "Marry had a little lamb." | knife 2-4
  Time (mean ± σ):       1.7 ms ±   0.5 ms    [User: 1.0 ms, System: 0.6 ms]
  Range (min … max):     1.0 ms …   4.5 ms    10000 runs
 
Summary
  echo "Marry had a little lamb." | knife 2-4 ran
    1.00 ± 0.38 times faster than echo "Marry had a little lamb." | cut -d" " -f 2-4

$ hyperfine -w 3 -N "cut -d' ' -f 2-10,50 IMDB\ Dataset.csv" "knife 2-10,50 IMDB\ Dataset.csv"
Benchmark 1: cut -d' ' -f 2-10,50 IMDB\ Dataset.csv
  Time (mean ± σ):     229.1 ms ±   1.1 ms    [User: 203.7 ms, System: 25.2 ms]
  Range (min … max):   227.4 ms … 231.4 ms    13 runs
 
Benchmark 2: knife 2-10,50 IMDB\ Dataset.csv
  Time (mean ± σ):     147.7 ms ±   1.7 ms    [User: 106.3 ms, System: 41.1 ms]
  Range (min … max):   146.0 ms … 152.3 ms    20 runs
 
Summary
  knife 2-10,50 IMDB\ Dataset.csv ran
    1.55 ± 0.02 times faster than cut -d' ' -f 2-10,50 IMDB\ Dataset.csv
```


 [`std::str::SplitWhitespace`]: https://doc.rust-lang.org/std/str/struct.SplitWhitespace.html
