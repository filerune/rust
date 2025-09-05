use std::{
    cell::RefCell, env, fs, hint::black_box, path::PathBuf, time::Duration,
};

use criterion::{
    Criterion,
    async_executor::{AsyncStdExecutor, SmolExecutor},
    criterion_group, criterion_main,
};
use filerune_fusion::{
    check::Check,
    merge::Merge,
    split::{Split, SplitResult},
};
use tokio::runtime::Runtime;

const RUNTIME_STD: &str = "std";
const RUNTIME_ASYNC_STD: &str = "async_std";
const RUNTIME_SMOL: &str = "smol";
const RUNTIME_TOKIO: &str = "tokio";

const FILE_NAME: &str = "test.jpg";

struct Configs {
    in_file: PathBuf,
    cache_dir: PathBuf,
    out_dir: PathBuf,
}

fn get_configs<R: Into<String>>(runtime: R) -> Configs {
    let root: PathBuf = env::current_dir().unwrap();

    let runtime: String = runtime.into();

    Configs {
        in_file: root.join("assets").join(FILE_NAME),
        cache_dir: root.join(".media").join("cache").join(&runtime),
        out_dir: root.join(".media").join("output").join(&runtime),
    }
}

fn pre_split<R: Into<String>>(runtime: R) -> SplitResult {
    let configs: Configs = get_configs(runtime);

    Split::new()
        .in_file(&configs.in_file)
        .out_dir(&configs.cache_dir.join("-1"))
        .run()
        .unwrap()
}

fn bench_split(c: &mut Criterion) {
    let mut group = c.benchmark_group("split");
    group.warm_up_time(Duration::from_secs(5));
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    group.bench_function("fusion_std", |b| {
        let configs: Configs = get_configs(RUNTIME_STD);

        if configs.out_dir.exists() {
            fs::remove_dir_all(&configs.out_dir).unwrap();
        }

        let mut i: usize = 0;

        b.iter(|| {
            let out_dir: PathBuf = configs.cache_dir.join(i.to_string());

            let result: SplitResult = Split::new()
                .in_file(&configs.in_file)
                .out_dir(out_dir)
                .run()
                .unwrap();

            black_box(result);

            i += 1;
        });
    });

    group.bench_function("fusion_async_std", |b| {
        use filerune_fusion::split::async_std::SplitAsyncExt as _;

        let configs: Configs = get_configs(RUNTIME_ASYNC_STD);

        if configs.out_dir.exists() {
            fs::remove_dir_all(&configs.out_dir).unwrap();
        }

        let i: RefCell<usize> = RefCell::new(0);

        b.to_async(AsyncStdExecutor).iter(async || {
            let idx: usize = {
                let mut borrow = i.borrow_mut();
                let val = *borrow;
                *borrow += 1;
                val
            };

            let out_dir: PathBuf = configs.cache_dir.join(idx.to_string());

            let result: SplitResult = Split::new()
                .in_file(&configs.in_file)
                .out_dir(out_dir)
                .run_async()
                .await
                .unwrap();

            black_box(result);
        });
    });

    group.bench_function("fusion_smol", |b| {
        use filerune_fusion::split::smol::SplitAsyncExt as _;

        let configs: Configs = get_configs(RUNTIME_SMOL);

        if configs.out_dir.exists() {
            fs::remove_dir_all(&configs.out_dir).unwrap();
        }

        let i: RefCell<usize> = RefCell::new(0);

        b.to_async(SmolExecutor).iter(async || {
            let idx: usize = {
                let mut borrow = i.borrow_mut();
                let val = *borrow;
                *borrow += 1;
                val
            };

            let out_dir: PathBuf = configs.cache_dir.join(idx.to_string());

            let result: SplitResult = Split::new()
                .in_file(&configs.in_file)
                .out_dir(out_dir)
                .run_async()
                .await
                .unwrap();

            black_box(result);
        });
    });

    group.bench_function("fusion_tokio", |b| {
        use filerune_fusion::split::tokio::SplitAsyncExt as _;

        let runtime: Runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let configs: Configs = get_configs(RUNTIME_TOKIO);

        if configs.out_dir.exists() {
            fs::remove_dir_all(&configs.out_dir).unwrap();
        }

        let i: RefCell<usize> = RefCell::new(0);

        b.to_async(runtime).iter(async || {
            let idx: usize = {
                let mut borrow = i.borrow_mut();
                let val = *borrow;
                *borrow += 1;
                val
            };

            let out_dir: PathBuf = configs.cache_dir.join(idx.to_string());

            let result: SplitResult = Split::new()
                .in_file(&configs.in_file)
                .out_dir(out_dir)
                .run_async()
                .await
                .unwrap();

            black_box(result);
        });
    });

    group.finish();
}

fn bench_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("check");
    group.warm_up_time(Duration::from_secs(5));

    group.bench_function("fusion_std", |b| {
        let split: SplitResult = pre_split(RUNTIME_STD);
        let configs: Configs = get_configs(RUNTIME_STD);

        b.iter(|| {
            let result: () = Check::new()
                .in_dir(&configs.cache_dir.join("0"))
                .file_size(split.file_size)
                .total_chunks(split.total_chunks)
                .run()
                .unwrap();

            black_box(result);
        });
    });

    group.bench_function("fusion_async_std", |b| {
        use filerune_fusion::check::async_std::CheckAsyncExt as _;

        let split: SplitResult = pre_split(RUNTIME_ASYNC_STD);
        let configs: Configs = get_configs(RUNTIME_ASYNC_STD);

        b.to_async(AsyncStdExecutor).iter(async || {
            let result: () = Check::new()
                .in_dir(&configs.cache_dir.join("0"))
                .file_size(split.file_size)
                .total_chunks(split.total_chunks)
                .run_async()
                .await
                .unwrap();

            black_box(result);
        });
    });

    group.bench_function("fusion_smol", |b| {
        use filerune_fusion::check::smol::CheckAsyncExt as _;

        let split: SplitResult = pre_split(RUNTIME_SMOL);
        let configs: Configs = get_configs(RUNTIME_SMOL);

        b.to_async(SmolExecutor).iter(async || {
            let result: () = Check::new()
                .in_dir(&configs.cache_dir.join("0"))
                .file_size(split.file_size)
                .total_chunks(split.total_chunks)
                .run_async()
                .await
                .unwrap();

            black_box(result);
        });
    });

    group.bench_function("fusion_tokio", |b| {
        use filerune_fusion::check::tokio::CheckAsyncExt as _;

        let split: SplitResult = pre_split(RUNTIME_TOKIO);
        let configs: Configs = get_configs(RUNTIME_TOKIO);

        let runtime: Runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        b.to_async(runtime).iter(async || {
            let result: () = Check::new()
                .in_dir(&configs.cache_dir.join("0"))
                .file_size(split.file_size)
                .total_chunks(split.total_chunks)
                .run_async()
                .await
                .unwrap();

            black_box(result);
        });
    });

    group.finish();
}

fn bench_merge(c: &mut Criterion) {
    let mut group = c.benchmark_group("merge");
    group.warm_up_time(Duration::from_secs(5));
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    group.bench_function("fusion_std", |b| {
        let configs: Configs = get_configs(RUNTIME_STD);

        if configs.out_dir.exists() {
            fs::remove_dir_all(&configs.out_dir).unwrap();
        }

        let mut i: usize = 0;

        b.iter(|| {
            let out_file: PathBuf = configs.out_dir.join(format!("{}.jpg", i));

            let result: () = Merge::new()
                .in_dir(&configs.cache_dir.join("0"))
                .out_file(out_file)
                .run()
                .unwrap();

            black_box(result);

            i += 1;
        });
    });

    group.bench_function("fusion_async_std", |b| {
        use filerune_fusion::merge::async_std::MergeAsyncExt as _;

        let configs: Configs = get_configs(RUNTIME_ASYNC_STD);

        if configs.out_dir.exists() {
            fs::remove_dir_all(&configs.out_dir).unwrap();
        }

        let i: RefCell<usize> = RefCell::new(0);

        b.to_async(AsyncStdExecutor).iter(async || {
            let idx: usize = {
                let mut borrow = i.borrow_mut();
                let val = *borrow;
                *borrow += 1;
                val
            };

            let out_file: PathBuf =
                configs.out_dir.join(format!("{}.jpg", idx));

            let result: () = Merge::new()
                .in_dir(&configs.cache_dir.join("0"))
                .out_file(out_file)
                .run_async()
                .await
                .unwrap();

            black_box(result);
        });
    });

    group.bench_function("fusion_smol", |b| {
        use filerune_fusion::merge::smol::MergeAsyncExt as _;

        let configs: Configs = get_configs(RUNTIME_SMOL);

        if configs.out_dir.exists() {
            fs::remove_dir_all(&configs.out_dir).unwrap();
        }

        let i: RefCell<usize> = RefCell::new(0);

        b.to_async(SmolExecutor).iter(async || {
            let idx: usize = {
                let mut borrow = i.borrow_mut();
                let val = *borrow;
                *borrow += 1;
                val
            };

            let out_file: PathBuf =
                configs.out_dir.join(format!("{}.jpg", idx));

            let result: () = Merge::new()
                .in_dir(&configs.cache_dir.join("0"))
                .out_file(out_file)
                .run_async()
                .await
                .unwrap();

            black_box(result);
        });
    });

    group.bench_function("fusion_tokio", |b| {
        use filerune_fusion::merge::tokio::MergeAsyncExt as _;

        let configs: Configs = get_configs(RUNTIME_TOKIO);

        if configs.out_dir.exists() {
            fs::remove_dir_all(&configs.out_dir).unwrap();
        }

        let i: RefCell<usize> = RefCell::new(0);

        let runtime: Runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        b.to_async(runtime).iter(async || {
            let idx: usize = {
                let mut borrow = i.borrow_mut();
                let val = *borrow;
                *borrow += 1;
                val
            };

            let out_file: PathBuf =
                configs.out_dir.join(format!("{}.jpg", idx));

            let result: () = Merge::new()
                .in_dir(&configs.cache_dir.join("0"))
                .out_file(out_file)
                .run_async()
                .await
                .unwrap();

            black_box(result);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    // split
    bench_split,
    // check
    bench_check,
    // merge
    bench_merge,
);
criterion_main!(benches);
