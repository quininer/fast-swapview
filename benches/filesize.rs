use criterion::{ criterion_main, criterion_group, Criterion, black_box };
use smallvec::SmallVec;
use fast_floats::Fast;


fn bench_fast_ryu_small(c: &mut Criterion) {
    fn filesize(size: isize) -> SmallVec<[u8; 8]> {
        const UNITS: [u8; 4] = [b'K', b'M', b'G', b'T'];

        fn take1(n: Fast<f64>) -> f64 {
            (n * 10.).get().round() / 10.
        }

        let mut left = Fast(size.abs() as f64);
        let mut unit = -1;

        while left > Fast(1100.) && unit < 3 {
            left = left / 1024.;
            unit += 1;
        }

        let mut output = SmallVec::new();
        if unit == -1 {
            output.extend_from_slice(itoa::Buffer::new().format(size).as_bytes());
            output.push(b'B');
        } else {
            if size < 0 {
                left = Fast(-left.get());
            }
            output.extend_from_slice(ryu::Buffer::new().format(take1(left)).as_bytes());
            output.push(UNITS[unit as usize]);
            output.extend_from_slice(b"iB");
        }
        output
    }

    c.bench_function("filesize-fast-ryu-small", move |b| {
        let (x, y, z): (isize, isize, isize) =
            black_box((0, 16080896, 231677952));

        b.iter(move || {
            black_box(filesize(x));
            black_box(filesize(y));
            black_box(filesize(z));
        });
    });
}


fn bench_fast_ryu(c: &mut Criterion) {
    fn filesize(size: isize) -> Box<str> {
        const UNITS: [char; 4] = ['K', 'M', 'G', 'T'];

        fn take1(n: Fast<f64>) -> f64 {
            (n * 10.).get().round() / 10.
        }

        let mut left = Fast(size.abs() as f64);
        let mut unit = -1;

        while left > Fast(1100.) && unit < 3 {
            left = left / 1024.;
            unit += 1;
        }

        let mut output = String::with_capacity(8);
        if unit == -1 {
            output.push_str(itoa::Buffer::new().format(size));
            output.push('B');
        } else {
            if size < 0 {
                left = Fast(-left.get());
            }
            output.push_str(ryu::Buffer::new().format(take1(left)));
            output.push(UNITS[unit as usize]);
            output.push_str("iB");
        }
        output.into_boxed_str()
    }

    c.bench_function("filesize-fast-ryu", move |b| {
        let (x, y, z): (isize, isize, isize) =
            black_box((0, 16080896, 231677952));

        b.iter(move || {
            black_box(filesize(x));
            black_box(filesize(y));
            black_box(filesize(z));
        });
    });
}

fn bench_ryu(c: &mut Criterion) {
    fn filesize(size: isize) -> Box<str> {
        const UNITS: [char; 4] = ['K', 'M', 'G', 'T'];

        fn take1(n: f64) -> f64 {
            (n * 10.).round() / 10.
        }

        let mut left = size.abs() as f64;
        let mut unit = -1;

        while left > 1100. && unit < 3 {
            left = left / 1024.;
            unit += 1;
        }

        let mut output = String::with_capacity(8);
        if unit == -1 {
            output.push_str(itoa::Buffer::new().format(size));
        } else {
            if size < 0 {
                left = -left;
            }
            output.push_str(ryu::Buffer::new().format(take1(left)));
            output.push(UNITS[unit as usize]);
            output.push_str("iB");
        }
        output.into_boxed_str()
    }

    c.bench_function("filesize-ryu", move |b| {
        let (x, y, z): (isize, isize, isize) =
            black_box((0, 16080896, 231677952));

        b.iter(move || {
            black_box(filesize(x));
            black_box(filesize(y));
            black_box(filesize(z));
        });
    });
}

fn bench_simple(c: &mut Criterion) {
    fn filesize(size: isize) -> String {
        const UNITS: [char; 4] = ['K', 'M', 'G', 'T'];

        let mut left = size.abs() as f64;
        let mut unit = -1;

        while left > 1100. && unit < 3 {
            left /= 1024.;
            unit += 1;
        }
        if unit == -1 {
            format!("{}B", size)
        } else {
            if size < 0 {
                left = -left;
            }
            format!("{:.1}{}iB", left, UNITS[unit as usize])
        }
    }

    c.bench_function("filesize-simple", move |b| {
        let (x, y, z): (isize, isize, isize) =
            black_box((0, 16080896, 231677952));

        b.iter(move || {
            black_box(filesize(x));
            black_box(filesize(y));
            black_box(filesize(z));
        });
    });
}

criterion_group!(filesize, bench_fast_ryu_small, bench_fast_ryu, bench_ryu, bench_simple);
criterion_main!(filesize);
