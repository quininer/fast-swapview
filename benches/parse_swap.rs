use std::str;
use std::io::{ BufRead, BufReader };
use criterion::{ criterion_main, criterion_group, Criterion, black_box };
use fast_lines::ReadLine;

const SMAPS: &[u8] = include_bytes!("smaps");
const LINE0: &[u8] = b"Rss:                   0 kB";
const LINE248: &[u8] = b"Rss:                 248 kB";
const LINE1948: &[u8] = b"Rss:                1948 kB";


fn nom_parse(input: &[u8]) -> Option<isize> {
    use nom::bytes::complete::{ tag, take_until };
    use nom::character::complete::multispace1;

    #[inline]
    fn foo(input: &[u8]) -> Result<isize, nom::Err<()>> {
        let (input, _) = tag("Swap:")(input)?;
        let (input, _) = multispace1(input)?;
        let (_, output) = take_until(" ")(input)?;
        let result = lexical_core::try_atoisize_slice(output);
        if let lexical_core::ErrorCode::Success = result.error.code {
            Ok(result.value)
        } else {
            Err(nom::Err::Failure(()))
        }
    }

    foo(input).ok()
}

fn simple_parse(input: &str) -> Option<isize> {
    if input.starts_with("Swap:") {
        input.split_whitespace().nth(1)?.parse::<isize>().ok()
    } else {
        None
    }
}

fn bench_parse_nom(c: &mut Criterion) {
    c.bench_function("parse-nom", move |b| {
        b.iter(move || {
            let _ = black_box(nom_parse(LINE0));
            let _ = black_box(nom_parse(LINE248));
            let _ = black_box(nom_parse(LINE1948));
        });
    });
}

fn bench_parse_simple(c: &mut Criterion) {
    c.bench_function("parse-simple", move |b| {
        b.iter(move || {
            let _ = black_box(simple_parse(str::from_utf8(LINE0).unwrap()));
            let _ = black_box(simple_parse(str::from_utf8(LINE248).unwrap()));
            let _ = black_box(simple_parse(str::from_utf8(LINE1948).unwrap()));
        });
    });
}

criterion_group!(parse_swap, bench_parse_nom, bench_parse_simple);


fn bench_read_swap_fast_nom(c: &mut Criterion) {
    c.bench_function("read-nom", move |b| {
        b.iter(move || {
            let mut s = 0;
            let mut reader = ReadLine::new(black_box(SMAPS));

            while let Ok(Some(line)) = reader.read_line() {
                if let Some(n) = nom_parse(line) {
                    s += n;
                }
            }

            black_box(s * 1024)
        });
    });
}

fn bench_read_swap_simple(c: &mut Criterion) {
    c.bench_function("read-simple", move |b| {
        b.iter(move || {
            let mut s = 0;
            let reader = BufReader::new(black_box(SMAPS));

            for l in reader.lines() {
                let line = match l {
                    Ok(s) => s,
                    Err(_) => break,
                };

                if let Some(n) = simple_parse(&line) {
                    s += n;
                }
            }

            black_box(s * 1024)
        });
    });
}

fn bench_read_swap_simple_buf(c: &mut Criterion) {
    c.bench_function("read-simple-buf", move |b| {
        b.iter(move || {
            let mut s = 0;
            let mut reader = BufReader::new(black_box(SMAPS));
            let mut line = Vec::new();

            while let Ok(n) = reader.read_until(b'\n', &mut line) {
                if n > 0 {
                    if let Ok(line) = str::from_utf8(&line) {
                        if let Some(n) = simple_parse(line) {
                            s += n;
                        }
                    }
                    line.clear();
                } else {
                    break;
                }
            }

            black_box(s * 1024)
        });
    });
}

criterion_group!(read_swap, bench_read_swap_fast_nom, bench_read_swap_simple, bench_read_swap_simple_buf);

fn bench_std_atoi(c: &mut Criterion) {
    fn atoi(input: &[u8]) -> Option<isize> {
        str::from_utf8(input).ok()?
            .parse().ok()
    }

    c.bench_function("std-atoi", move |b| {
        b.iter(|| {
            black_box(atoi(black_box(b"0")));
            black_box(atoi(black_box(b"248")));
            black_box(atoi(black_box(b"1948")));
        });
    });
}

fn bench_lexical_atoi(c: &mut Criterion) {
    fn atoi(input: &[u8]) -> Option<isize> {
        let result = lexical_core::try_atoisize_slice(input);
        if let lexical_core::ErrorCode::Success = result.error.code {
            Some(result.value)
        } else {
            None
        }
    }

    c.bench_function("lexical-atoi", move |b| {
        b.iter(|| {
            black_box(atoi(black_box(b"0")));
            black_box(atoi(black_box(b"248")));
            black_box(atoi(black_box(b"1948")));
        });
    });
}

criterion_group!(atoi, bench_std_atoi, bench_lexical_atoi);
criterion_main!(parse_swap, read_swap, atoi);
