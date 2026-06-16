use mnemonic_engrave::convert;

#[test]
fn md1_short_matches_golden() {
    let golden = include_bytes!("vectors/md1-short.ndef");
    let got = convert("md1yqpqqxqq8xtwhw4xwn4qh").unwrap();
    assert_eq!(&got[..], &golden[..]);
}
