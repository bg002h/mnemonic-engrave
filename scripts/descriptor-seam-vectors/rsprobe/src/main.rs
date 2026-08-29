// Cross-check probe: reassemble an md1 set with the PUBLISHED md-codec 0.42 --
// the exact crate `me` links -- and print its wallet_policy_id. Reads the md1
// strings on stdin, one per line.
use std::io::Read as _;

fn main() {
    let mut s = String::new();
    std::io::stdin().read_to_string(&mut s).unwrap();
    let strs: Vec<&str> = s.split_whitespace().filter(|l| !l.is_empty()).collect();
    let d = md_codec::reassemble(&strs).expect("reassemble");
    let id = md_codec::compute_wallet_policy_id(&d).expect("wallet policy id");
    println!("{}", hex(id.as_bytes()));
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
