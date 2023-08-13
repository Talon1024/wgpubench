use futures::executor;

fn main() {
    executor::block_on(wgpubench::run());
}
