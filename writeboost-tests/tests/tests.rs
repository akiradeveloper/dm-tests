use writeboost_tests::*;
use device_mapper_tests::*;

#[test]
fn test_rw() {
    let mut env = env();
    let slow = env.alloc_device(Sector::MB(500));
    let fast = env.alloc_device(Sector::MB(100));
    sweep_caches(&fast);
    let table = Table {
        backing_dev: slow.path(),
        cache_dev: fast.path(),
        options: Options::default(),
    };
    let dev = Writeboost::create(table);
    io::test_blk_rw(&dev, Sector(0), Sector::KB(1));
    io::test_blk_rw(&dev, Sector(0), Sector::MB(50));
    io::test_blk_rw(&dev, Sector(0), Sector::MB(200));
}

#[test]
fn test_stat() {
    let mut env = env();
    let slow = env.alloc_device(Sector::MB(500));
    let fast = env.alloc_device(Sector::MB(100));
    sweep_caches(&fast);
    let table = Table {
        backing_dev: slow.path(),
        cache_dev: fast.path(),
        options: Options::default(),
    };
    let dev = Writeboost::create(table);

    // After create
    let stat0 = dev.status().stat;
    for i in 0..16 {
        let k = StatKey::from_bits(i);
        let v = *stat0.get(&k).unwrap();
        assert_eq!(v, 0);
    }

    // After open
    let rw = open(&dev);
    let stat1 = dev.status().stat;
    for i in 0..16 {
        let k = StatKey::from_bits(i);
        let v = *stat1.get(&k).unwrap();
        assert_eq!(v, 0);
    }
}