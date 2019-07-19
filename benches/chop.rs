use criterion::{ criterion_main, criterion_group, Criterion, black_box };
use bstr::{ ByteVec, ByteSlice };


const CMD1: &str = "python3\0-c\0import sys; sys.path.remove(\"\"); import neovim; neovim.start_host()\0script_host.py\0";
const CMD2: &str = "tmux\0";
const CMD3: &str = "vi\0src/main.rs\0";

fn bench_copyless(c: &mut Criterion) {
    pub fn chop_null(mut s: Vec<u8>) -> Vec<u8> {
        if let Some(0x00) = s.as_bytes().last_byte() {
            s.pop_byte();
        }

        for b in s.as_bytes_mut() {
            if *b == 0x0 {
                *b = b' ';
            }
        }

        s
    }

    c.bench_function("null-copyless", move |b| {
        b.iter(|| {
            black_box(chop_null(black_box(CMD1.into())));
            black_box(chop_null(black_box(CMD2.into())));
            black_box(chop_null(black_box(CMD3.into())));
        })
    });
}

fn bench_simple(c: &mut Criterion) {
    pub fn chop_null(s: String) -> String {
        let last = s.len() - 1;
        let mut s = s;
        if !s.is_empty() && s.as_bytes()[last] == 0 {
            s.truncate(last);
        }
        s.replace("\0", " ")
    }

    c.bench_function("null-simple", move |b| {
        b.iter(|| {
            black_box(chop_null(black_box(CMD1.to_string())));
            black_box(chop_null(black_box(CMD2.to_string())));
            black_box(chop_null(black_box(CMD3.to_string())));
        })
    });
}

criterion_group!(chop, bench_copyless, bench_simple);
criterion_main!(chop);
