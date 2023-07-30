fn main() {
    pollster::block_on(wgpubench::run());
}
