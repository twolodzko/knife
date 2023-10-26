# It's just a sharper knife 🔪

`knife` is like the `cut` command, but delimits fields with whitespaces, any whitespaces.

If you ever felt frustrated when trying to extract whitespace delimited fields? _Should I split on tabs or spaces? How many?_

```shell
$ echo "Mary   had a     little  lamb." | cut -f 2-4   
Mary   had a     little  lamb.
$ echo "Mary   had a     little  lamb." | cut -d' ' -f 2-4
  had
```

I created `knife` exactly for this purpose. It splits the input strings by whitespaces using Rust's [`std::str::SplitWhitespace`]
and extracts fields specified using a pattern language like the one used by `cut`.

```shell
$ echo "Mary   had a     little  lamb." | knife 2-4   
had a little
```

That's it, it doesn't do anything more. If you need more customizable search, use dedicated tools like
`grep`, `sed`, `awk`, etc or the `cut` itself.

## Installation

To install it run:

```shell
cargo install --git https://github.com/twolodzko/knife.git
```

## Benchmarks

`knife` can be faster than `cut` in some common scenarios and slower or equal in others. While the code is optimized in
several places, it was not build for speed.

```shell
$ hyperfine -N -r 10000 \
 'echo "Marry had a little lamb." | cut -d" " -f 2-4' \
 'echo "Marry had a little lamb." | knife 2-4'
Benchmark 1: echo "Marry had a little lamb." | cut -d" " -f 2-4
  Time (mean ± σ):       1.5 ms ±   0.2 ms    [User: 1.0 ms, System: 0.4 ms]
  Range (min … max):     1.1 ms …   5.0 ms    10000 runs
 
  Warning: The first benchmarking run for this command was significantly slower than the rest (4.3 ms). This could be caused by (filesystem) caches that were not filled until after the first run. You should consider using the '--warmup' option to fill those caches before the actual benchmark. Alternatively, use the '--prepare' option to clear the caches before each timing run.
 
Benchmark 2: echo "Marry had a little lamb." | knife 2-4
  Time (mean ± σ):       1.5 ms ±   0.1 ms    [User: 1.0 ms, System: 0.4 ms]
  Range (min … max):     1.1 ms …   3.7 ms    10000 runs
 
  Warning: Statistical outliers were detected. Consider re-running this benchmark on a quiet system without any interferences from other programs. It might help to use the '--warmup' or '--prepare' options.
 
Summary
  echo "Marry had a little lamb." | knife 2-4 ran
    1.00 ± 0.17 times faster than echo "Marry had a little lamb." | cut -d" " -f 2-4


$ hyperfine -N \
  "cut -d' ' -f 2-5,20 IMDB\ Dataset.csv" \
  "knife 2-5,20 IMDB\ Dataset.csv"                                   
Benchmark 1: cut -d' ' -f 2-5,20 IMDB\ Dataset.csv
  Time (mean ± σ):     252.0 ms ±  48.8 ms    [User: 211.9 ms, System: 38.4 ms]
  Range (min … max):   233.6 ms … 390.8 ms    10 runs
 
  Warning: The first benchmarking run for this command was significantly slower than the rest (390.8 ms). This could be caused by (filesystem) caches that were not filled until after the first run. You should consider using the '--warmup' option to fill those caches before the actual benchmark. Alternatively, use the '--prepare' option to clear the caches before each timing run.
 
Benchmark 2: knife 2-5,20 IMDB\ Dataset.csv
  Time (mean ± σ):     123.3 ms ±   2.4 ms    [User: 74.8 ms, System: 48.2 ms]
  Range (min … max):   119.5 ms … 130.3 ms    23 runs
 
Summary
  knife 2-5,20 IMDB\ Dataset.csv ran
    2.04 ± 0.40 times faster than cut -d' ' -f 2-5,20 IMDB\ Dataset.csv
```


 [`std::str::SplitWhitespace`]: https://doc.rust-lang.org/std/str/struct.SplitWhitespace.html
