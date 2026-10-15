fn main() {
    lalrpop::process_root().unwrap();
    fn files(root:&std::path::Path, all:&mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(root).unwrap() {
            let path=entry.unwrap().path();
            if path.is_dir() { files(&path,all); }
            else if path.extension().is_some_and(|s| s=="rs") { all.push(path); }
        }
    }
    let mut paths=Vec::new();
    files(std::path::Path::new("src"),&mut paths);
    files(std::path::Path::new("tiger-tables/src/ck3"),&mut paths);
    paths.sort();
    let mut hash=0xcbf29ce484222325u64;
    for path in paths {
        println!("cargo:rerun-if-changed={}",path.display());
        for byte in path.to_string_lossy().bytes().chain(std::fs::read(&path).unwrap()) { hash=(hash ^ u64::from(byte)).wrapping_mul(0x100000001b3); }
    }
    println!("cargo:rustc-env=TIGER_RULES_FINGERPRINT={hash:016x}");
}
