// Author: Dylan Jones
// Date:   2025-11-11

use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::options::benches,
    benchmarks::anlz::benches,
}
