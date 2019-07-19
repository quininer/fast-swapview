use std::sync::mpsc::channel;
use criterion::{ criterion_main, criterion_group, Criterion, black_box };
use rand::{ Rng, SeedableRng, distributions::Alphanumeric, rngs::SmallRng };
use crossbeam_skiplist::SkipMap;
use rayon::prelude::*;

#[derive(Clone)]
struct DataIter {
    pid: usize,
    rng: SmallRng
}

impl DataIter {
    fn new() -> DataIter {
        DataIter { pid: 0, rng: SmallRng::from_entropy() }
    }
}

impl Iterator for DataIter {
    type Item = (usize, isize, String);

    fn next(&mut self) -> Option<Self::Item> {
        self.pid += 1;
        let size = self.rng.gen();
        let comm = (0..self.rng.gen_range(1, 33))
            .map(|_| self.rng.sample(Alphanumeric))
            .collect::<String>();
        Some((self.pid, size, comm))
    }
}

fn bench_merge_vec_sort(c: &mut Criterion) {
    c.bench_function("merge-vec-sort", move |b| {
        let iter = black_box(DataIter::new());

        b.iter(move || {
            let iter = black_box(iter.clone());
            let mut info = iter
                .take(100)
                .par_bridge()
                .map(|info| black_box(info))
                .collect::<Vec<_>>();
            info.sort_unstable_by_key(|&(_, size, _)| size);
            for (pid, swap, comm) in info {
                black_box((pid, swap, comm));
            }
        });
    });
}

fn bench_merge_channel(c: &mut Criterion) {
    c.bench_function("merge-channel", move |b| {
        let iter = black_box(DataIter::new());

        b.iter(move || {
            let iter = black_box(iter.clone());

            let mut info = rayon::scope(|pool| {
                let (tx, rx) = channel();
                for info in iter.take(100) {
                    let tx = tx.clone();
                    pool.spawn(move |_| {
                        let (pid, swap, comm) = black_box(info);
                        let _ = tx.send(Some((pid, swap, comm)));
                    });
                }
                let _ = tx.send(None);
                drop(tx);
                rx.iter().filter_map(|x| x).collect::<Vec<_>>()
            });

            info.par_sort_unstable_by_key(|&(_, size, _)| size);

            for (pid, swap, comm) in info {
                black_box((pid, swap, comm));
            }
        });
    });
}

fn bench_merge_skiplist(c: &mut Criterion) {
    c.bench_function("merge-skiplist", move |b| {
        let iter = black_box(DataIter::new());

        b.iter(move || {
            let iter = black_box(iter.clone());
            let info = SkipMap::new();
            iter
                .take(100)
                .par_bridge()
                .map(|info| black_box(info))
                .for_each(|(pid, swap, comm)| {
                    info.insert(swap, (pid, swap, comm));
                });

            // slow ...
            for (_, (pid, swap, comm)) in info {
                black_box((pid, swap, comm));
            }
        });
    });
}

criterion_group!(par_merge, bench_merge_vec_sort, bench_merge_channel, bench_merge_skiplist);
criterion_main!(par_merge);
