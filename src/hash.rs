use ring::digest::{Context, SHA256};

pub fn digest(s: &str) -> String {
    let mut ctx = Context::new(&SHA256);
    ctx.update(s.as_bytes());
    data_encoding::HEXLOWER.encode(ctx.finish().as_ref())
}