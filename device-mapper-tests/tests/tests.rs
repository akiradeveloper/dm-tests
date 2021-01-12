use device_mapper_tests::*;
use cmd_lib::{run_cmd, run_fun};

#[test]
fn test_kernel() {
    kernel::drop_caches();
}
#[test]
fn test_sector() {
    let x = Sector::MB(5);
    let y = Sector::MB(3);
    assert_eq!(x+y, Sector::MB(8));
    assert_eq!(x-y, Sector::KB(2048));
    assert_eq!(x*4, Sector::MB(20));
}
#[test]
fn test_mem_pool() {
    let mut env = env();
    let mut xs = vec![];
    for _ in 0..10 {
        xs.push(env.alloc_device(Sector::MB(1)));
    }
}
#[test]
fn test_blkdev_size() {
    let mut env = env();
    let dev = env.alloc_device(Sector(127));
    let sz = blkdev::getsize(&dev.path());
    assert_eq!(sz, Sector(127));
}
fn test_blk_rw(s: &impl Stack) {
    let rw = open(s);
    let wbuf = vec![1;512];
    rw.write(&wbuf, Sector::KB(1), Sector(1));
    let mut rbuf = vec![0;512];
    rw.read(&mut rbuf, Sector::KB(1), Sector(1));
    assert_eq!(rbuf, wbuf);
}
#[test]
fn test_mem_rw() {
    let mut env = env();
    let dev = env.alloc_device(Sector::MB(1));
    test_blk_rw(&dev)
}
#[test]
fn test_pattern_io() {
    use io::pattern::{PatternIO, Pattern};
    let mut env = env();
    let dev = env.alloc_device(Sector::GB(1));
    let rw = open(&dev);
    let pats = vec![
        Pattern::Write(Sector::MB(2)),
        Pattern::Skip(Sector::MB(3)),
        Pattern::Read(Sector::MB(2)),
        Pattern::Skip(Sector::MB(3)),
    ];
    let pat = PatternIO::new().max_io_amount(Sector::MB(500));
    pat.submit(&rw, &pats);
}
#[test]
fn test_linear() {
    use stack::linear::*;
    let mut env = env();
    let dev = env.alloc_device(Sector::MB(100));
    let dm = EmptyDMStack::new();
    let tab = Table {
        backing_dev: dev.path(),
        offset: Sector::MB(20),
        len: Sector::MB(40),
    };
    let linear = Linear::new(dm, tab);
    let sz = blkdev::getsize(&linear.path()); 
    assert_eq!(sz, Sector::MB(40));
    test_blk_rw(&linear);
}
#[test]
fn test_luks_prerequisite() {
    run_cmd!(cryptsetup benchmark -c aes-xts-plain64).unwrap()
}
#[test]
fn test_luks() {
    use stack::luks::*;
    let mut env = env();
    let dev = env.alloc_device(Sector::MB(100));
    Luks::format(&dev);
    let luks = Luks::new(&dev);
    test_blk_rw(&luks);
}
#[test]
fn test_flakey() {
    use stack::flakey::*;
    let mut env = env();
    let dev = env.alloc_device(Sector::MB(100));
    let dm = EmptyDMStack::new();
    let tab = Table {
        backing_dev: dev.path(),
        up_interval_sec: 8,
        down_interval_sec: 2,
    };
    let flakey = Flakey::new(dm, tab);
    // Should finish in 8 seconds
    test_blk_rw(&flakey);
}
#[test]
fn test_xfs_prerequisite() {
    run_cmd!(which mkfs.xfs).unwrap();
    run_cmd!(which xfs_repair).unwrap();
}
#[test]
fn test_xfs() {
    use fs::xfs::*;
    let mut env = env();
    let dev = env.alloc_device(Sector::MB(100));
    XFS::format(&dev);
    let fs = XFS::new(&dev);
    let fp = format!("{}/file", fs.mount_point);
    run_cmd!(touch $fp).unwrap();
    run_cmd!(rm $fp).unwrap();
}