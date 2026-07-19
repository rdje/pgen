#![crate_type = "lib"]
pub struct P<'a> { pub input: &'a str, pub position: usize }

#[inline(never)]
pub fn old<'a, const N: usize>(p: &mut P<'a>, expected: &'static str, eb: &[u8; N]) -> Result<&'a str, usize> {
    let start = p.position;
    let end = start + N;
    if end <= p.input.len() && p.input.as_bytes()[start..end] == *eb {
        p.position = end;
        return Ok(&p.input[start..end]);
    }
    Err(start)
}

#[inline(never)]
pub fn new<'a, const N: usize>(p: &mut P<'a>, expected: &'static str, eb: &[u8; N]) -> Result<&'a str, usize> {
    let start = p.position;
    let end = start + N;
    if end <= p.input.len() && p.input.as_bytes()[start..end] == *eb {
        p.position = end;
        return Ok(expected);
    }
    Err(start)
}

#[inline(never)] pub fn call_old<'a>(p: &mut P<'a>) -> Result<&'a str, usize> { old(p, "A", b"A") }
#[inline(never)] pub fn call_new<'a>(p: &mut P<'a>) -> Result<&'a str, usize> { new(p, "A", b"A") }
