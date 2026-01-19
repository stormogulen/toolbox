// benches/access_patterns.rs

use bytemuck_derive::{Pod, Zeroable};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use raw_bytes::Container;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Point3D {
    x: f64,
    y: f64,
    z: f64,
}

fn create_container(size: usize) -> Container<Point3D> {
    let mut c = Container::with_capacity(size);
    for i in 0..size {
        c.push(Point3D {
            x: i as f64,
            y: (i * 2) as f64,
            z: (i * 3) as f64,
        })
        .unwrap();
    }
    c
}

fn bench_individual_get(c: &mut Criterion) {
    let sizes = vec![100, 1_000, 10_000];

    let mut group = c.benchmark_group("individual_get");
    for size in sizes {
        let container = create_container(size);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let mut sum = 0.0;
                for i in 0..container.len() {
                    sum += black_box(container.get(i).unwrap().x);
                }
                sum
            });
        });
    }
    group.finish();
}

fn bench_iterator(c: &mut Criterion) {
    let sizes = vec![100, 1_000, 10_000];

    let mut group = c.benchmark_group("iterator");
    for size in sizes {
        let container = create_container(size);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let sum: f64 = container.iter().map(|p| black_box(p.x)).sum();
                sum
            });
        });
    }
    group.finish();
}

fn bench_slice_access(c: &mut Criterion) {
    let sizes = vec![100, 1_000, 10_000];

    let mut group = c.benchmark_group("slice_access");
    for size in sizes {
        let container = create_container(size);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let sum: f64 = container.as_slice().iter().map(|p| black_box(p.x)).sum();
                sum
            });
        });
    }
    group.finish();
}

fn bench_index_syntax(c: &mut Criterion) {
    let sizes = vec![100, 1_000, 10_000];

    let mut group = c.benchmark_group("index_syntax");
    for size in sizes {
        let container = create_container(size);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let mut sum = 0.0;
                for i in 0..container.len() {
                    sum += black_box(container[i].x);
                }
                sum
            });
        });
    }
    group.finish();
}

fn bench_write_operations(c: &mut Criterion) {
    let sizes = vec![100, 1_000, 10_000];

    let mut group = c.benchmark_group("write_operations");
    for size in sizes {
        group.bench_with_input(BenchmarkId::new("write_method", size), &size, |b, &s| {
            let mut container = create_container(s);
            b.iter(|| {
                for i in 0..container.len() {
                    container
                        .write(
                            i,
                            Point3D {
                                x: 1.0,
                                y: 2.0,
                                z: 3.0,
                            },
                        )
                        .unwrap();
                }
            });
        });

        group.bench_with_input(BenchmarkId::new("get_mut", size), &size, |b, &s| {
            let mut container = create_container(s);
            b.iter(|| {
                for i in 0..container.len() {
                    let p = container.get_mut(i).unwrap();
                    p.x = 1.0;
                    p.y = 2.0;
                    p.z = 3.0;
                }
            });
        });

        group.bench_with_input(BenchmarkId::new("mut_slice", size), &size, |b, &s| {
            let mut container = create_container(s);
            b.iter(|| {
                let slice = container.as_mut_slice().unwrap();
                for p in slice.iter_mut() {
                    p.x = 1.0;
                    p.y = 2.0;
                    p.z = 3.0;
                }
            });
        });
    }
    group.finish();
}

fn bench_push_operations(c: &mut Criterion) {
    let sizes = vec![100, 1_000, 10_000];

    let mut group = c.benchmark_group("push_operations");
    for size in sizes {
        group.bench_with_input(
            BenchmarkId::new("without_capacity", size),
            &size,
            |b, &s| {
                b.iter(|| {
                    let mut container = Container::<Point3D>::new();
                    for i in 0..s {
                        container
                            .push(Point3D {
                                x: i as f64,
                                y: (i * 2) as f64,
                                z: (i * 3) as f64,
                            })
                            .unwrap();
                    }
                });
            },
        );

        group.bench_with_input(BenchmarkId::new("with_capacity", size), &size, |b, &s| {
            b.iter(|| {
                let mut container = Container::<Point3D>::with_capacity(s);
                for i in 0..s {
                    container
                        .push(Point3D {
                            x: i as f64,
                            y: (i * 2) as f64,
                            z: (i * 3) as f64,
                        })
                        .unwrap();
                }
            });
        });

        group.bench_with_input(BenchmarkId::new("from_slice", size), &size, |b, &s| {
            let data: Vec<Point3D> = (0..s)
                .map(|i| Point3D {
                    x: i as f64,
                    y: (i * 2) as f64,
                    z: (i * 3) as f64,
                })
                .collect();

            b.iter(|| Container::from_slice(&data));
        });
    }
    group.finish();
}

#[cfg(feature = "mmap")]
fn bench_mmap_operations(c: &mut Criterion) {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let sizes = vec![100, 1_000, 10_000];

    let mut group = c.benchmark_group("mmap_operations");
    for size in sizes {
        // Create test file
        let data: Vec<Point3D> = (0..size)
            .map(|i| Point3D {
                x: i as f64,
                y: (i * 2) as f64,
                z: (i * 3) as f64,
            })
            .collect();

        let mut file = NamedTempFile::new().unwrap();
        let bytes: &[u8] = bytemuck::cast_slice(&data);
        file.write_all(bytes).unwrap();
        file.flush().unwrap();

        // Benchmark readonly
        group.bench_with_input(BenchmarkId::new("readonly_iter", size), &size, |b, _| {
            let container = Container::<Point3D>::mmap_readonly(file.path()).unwrap();
            b.iter(|| {
                let sum: f64 = container.iter().map(|p| black_box(p.x)).sum();
                sum
            });
        });

        // Benchmark readwrite
        group.bench_with_input(BenchmarkId::new("readwrite_read", size), &size, |b, _| {
            let container = Container::<Point3D>::mmap_readwrite(file.path()).unwrap();
            b.iter(|| {
                let sum: f64 = container.iter().map(|p| black_box(p.x)).sum();
                sum
            });
        });

        group.bench_with_input(BenchmarkId::new("readwrite_write", size), &size, |b, _| {
            let mut container = Container::<Point3D>::mmap_readwrite(file.path()).unwrap();
            b.iter(|| {
                let slice = container.as_mut_slice().unwrap();
                for p in slice.iter_mut() {
                    p.x = 999.0;
                }
            });
        });
    }
    group.finish();
}

fn bench_cache_effects(c: &mut Criterion) {
    use rand::prelude::*;
    
    let size = 10_000;
    let mut group = c.benchmark_group("cache_effects");
    
    // Sequential access (cache-friendly)
    group.bench_function("sequential", |b| {
        let container = create_container(size);
        b.iter(|| {
            let mut sum = 0.0;
            for i in 0..container.len() {
                sum += black_box(container[i].x);
            }
            sum
        });
    });
    
    // Random access (cache-unfriendly)
    group.bench_function("random", |b| {
        let container = create_container(size);
        let mut rng = StdRng::seed_from_u64(42);
        let indices: Vec<usize> = (0..size).map(|_| rng.gen_range(0..size)).collect();
        
        b.iter(|| {
            let mut sum = 0.0;
            for &i in &indices {
                sum += black_box(container[i].x);
            }
            sum
        });
    });
    
    group.finish();
}

fn bench_memory_footprint(c: &mut Criterion) {
    let sizes = vec![1_000, 10_000, 100_000];
    
    for size in sizes {
        println!("\n=== Size: {} elements ===", size);
        
        // In-memory
        let container = create_container(size);
        let vec_overhead = std::mem::size_of_val(&container);
        let data_size = size * std::mem::size_of::<Point3D>();
        println!("In-memory: {} bytes data + {} bytes overhead", 
                 data_size, vec_overhead);
        
        #[cfg(feature = "mmap")]
        {
            use std::io::Write;
            use tempfile::NamedTempFile;
            
            let data: Vec<Point3D> = (0..size)
                .map(|i| Point3D { x: i as f64, y: 0.0, z: 0.0 })
                .collect();
            
            let mut file = NamedTempFile::new().unwrap();
            file.write_all(bytemuck::cast_slice(&data)).unwrap();
            file.flush().unwrap();
            
            let mmap_container = Container::<Point3D>::mmap_readonly(file.path()).unwrap();
            let mmap_overhead = std::mem::size_of_val(&mmap_container);
            println!("Mmap: {} bytes data + {} bytes overhead", 
                     data_size, mmap_overhead);
        }
    }
}

fn bench_vs_raw_vec(c: &mut Criterion) {
    let size = 10_000;
    let mut group = c.benchmark_group("vs_raw_vec");
    
    // Container
    group.bench_function("container_iter", |b| {
        let container = create_container(size);
        b.iter(|| {
            container.iter().map(|p| black_box(p.x)).sum::<f64>()
        });
    });
    
    // Raw Vec
    group.bench_function("raw_vec_iter", |b| {
        let vec: Vec<Point3D> = (0..size)
            .map(|i| Point3D { x: i as f64, y: 0.0, z: 0.0 })
            .collect();
        b.iter(|| {
            vec.iter().map(|p| black_box(p.x)).sum::<f64>()
        });
    });
    
    group.finish();
}

criterion_group!(
    read_benches,
    bench_individual_get,
    bench_iterator,
    bench_slice_access,
    bench_index_syntax
);

criterion_group!(write_benches, bench_write_operations, bench_push_operations);

#[cfg(feature = "mmap")]
criterion_group!(mmap_benches, bench_mmap_operations);

// Register benchmark groups
#[cfg(feature = "mmap")]
criterion_main!(read_benches, write_benches, mmap_benches);

#[cfg(not(feature = "mmap"))]
criterion_main!(read_benches, write_benches);
