use std::time::Instant;

use av1_test::encoding::test_encoding;

fn main() {
    let now = Instant::now();
    test_encoding();
    dbg!(now.elapsed());
}
