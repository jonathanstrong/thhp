#[macro_use]
extern crate error_chain;

mod errors;

pub use errors::*;

pub struct Request<'buffer, 'header>
where
    'buffer: 'header,
{
    pub method: &'buffer [u8],
    pub target: &'buffer [u8],
    pub version: &'buffer [u8],
    pub headers: &'header Vec<HeaderField<'buffer>>,
}

pub struct HeaderField<'buffer> {
    pub name: &'buffer [u8],
    pub value: &'buffer [u8],
}

#[inline]
fn is_tchar(c: u8) -> bool {
    0x20 <= c && c <= 0x7E
}

#[inline]
fn is_digit(c: u8) -> bool {
    b'0' <= c && c <= b'9'
}

#[inline]
fn position<P>(buf: &[u8], mut predicate: P) -> Result<usize>
where
    P: FnMut(&u8) -> bool,
{
    let mut i = 0;
    loop {
        if let Some(c) = buf.get(i) {
            if predicate(c) {
                return Ok(i);
            }
            i += 1;
        } else {
            return Err(InvalidHeaderFormat.into());
        }
    }
}

#[inline]
fn read_until<D, A>(buf: &[u8], mut delimitor: D, mut acceptable: A) -> Result<usize>
where
    D: FnMut(&u8) -> bool,
    A: FnMut(&u8) -> bool,
{
    let mut i = 0;
    loop {
        match buf.get(i) {
            Some(c) => {
                if delimitor(c) {
                    return Ok(i);
                }

                if !acceptable(c) {
                    return Err(InvalidHeaderFormat.into());
                }

                i += 1;
            }
            None => return Err(InvalidHeaderFormat.into()),
        }
    }
}

#[inline]
fn parse_token(buf: &[u8]) -> Result<usize> {
    read_until(buf, |&x| x == b' ', |&x| is_tchar(x))
}

#[inline]
fn parse_http_version(buf: &[u8]) -> Result<usize> {
    if buf.len() < 3 {
        return Err(Incomplete.into());
    }

    if buf[0] == b'1' && buf[1] == b'.' && is_digit(buf[2]) {
        return Ok(3);
    } else {
        return Err(InvalidHeaderFormat.into());
    }
}

impl<'buffer, 'header> Request<'buffer, 'header> {
    pub fn parse(
        buf: &'buffer [u8],
        headers: &'header mut Vec<HeaderField<'buffer>>,
    ) -> Result<Request<'buffer, 'header>> {
        let mut s = 0;
        let mut i = 0;

        i += parse_token(buf)?;
        let method = unsafe { buf.get_unchecked(s..i) };

        i += 1;
        s = i;

        i += parse_token(&buf[i..])?;
        let target = unsafe { buf.get_unchecked(s..i) };

        i += 1;

        if !buf.get(i..).unwrap().starts_with(b"HTTP/") {
            return Err(ErrorKind::InvalidHeaderFormat.into());
        }

        i += 5;
        s = i;

        i += parse_http_version(&buf[i..])?;
        let version = &buf.get(s..i).unwrap();

        i += 1;
        i += 1; // '\n'

        parse_headers(&buf[i..], headers)?;

        return Ok(Request::<'buffer, 'header> {
            method: method,
            target: target,
            version: version,
            headers: headers,
        });
    }
}

pub fn parse_headers<'buffer, 'header>(
    buf: &'buffer [u8],
    result: &'header mut Vec<HeaderField<'buffer>>,
) -> Result<()> {
    if buf.len() == 0 {
        return Err(ErrorKind::Incomplete.into());
    }

    let mut s;
    let mut i = 0;
    loop {
        if buf[i] == b'\r' {
            break;
        }

        s = i;

        i += position(&buf[i..], |&x| x == b':')?;
        let name = s..i;

        i += 1;

        s = i;
        i += position(&buf[i..], |&x| x == b'\r')?;
        let value = s..i;

        i += 1;

        result.push(HeaderField::<'buffer> {
            name: &buf[name],
            value: &buf[value],
        });

        if buf[i] == b'\n' {
            i += 1;
        }
    }

    return Ok(());
}

#[cfg(test)]
mod tests {
    use ::*;

    #[test]
    fn empty_request_is_unparsable() {
        let mut headers = Vec::<HeaderField>::with_capacity(10);
        let result = Request::parse(b"", &mut headers);
        assert!(result.is_err());
    }

    #[test]
    fn parse_get_request() {
        let mut headers = Vec::<HeaderField>::with_capacity(10);
        let result = Request::parse(b"GET / HTTP/1.1\r\nname:value\r\n\r\n", &mut headers);
        assert!(result.is_ok());
        let req = result.unwrap();
        assert_eq!(req.method, b"GET");
        assert_eq!(req.target, b"/");
        assert_eq!(req.version, b"1.1");
        assert_eq!(req.headers.len(), 1);
        assert_eq!(req.headers[0].name, b"name");
        assert_eq!(req.headers[0].value, b"value");
    }

    #[test]
    fn parse_a_header_field() {
        let mut headers = Vec::<HeaderField>::with_capacity(10);
        let result = parse_headers(b"name:value\r\n\r\n", &mut headers);
        assert!(result.is_ok());
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].name, b"name");
        assert_eq!(headers[0].value, b"value");
    }

    #[test]
    fn parse_2_header_fields() {
        let mut headers = Vec::<HeaderField>::with_capacity(10);
        let result = parse_headers(b"name1:value1\r\nname2:value2\r\n\r\n", &mut headers);
        assert!(result.is_ok());
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0].name, b"name1");
        assert_eq!(headers[0].value, b"value1");
        assert_eq!(headers[1].name, b"name2");
        assert_eq!(headers[1].value, b"value2");
    }

    fn fail(r: Result<Request>, err: ErrorKind) {
        assert!(r.is_err());
        assert_eq!(*r.err().unwrap().kind(), err);
    }

    #[test]
    fn failures() {
        let mut headers = Vec::<HeaderField>::with_capacity(10);
        fail(
            Request::parse(b"G\x01ET / HTTP/1.1\r\n\r\n", &mut headers),
            InvalidHeaderFormat,
        );
        fail(
            Request::parse(b"G1ET /a\x01ef HTTP/1.1\r\n\r\n", &mut headers),
            InvalidHeaderFormat,
        );
        fail(
            Request::parse(b"G1ET / HOGE\r\n\r\n", &mut headers),
            InvalidHeaderFormat,
        );
        fail(
            Request::parse(b"G1ET / HTTP/11.1\r\n\r\n", &mut headers),
            InvalidHeaderFormat,
        );
        fail(
            Request::parse(b"G1ET / HTTP/A.1\r\n\r\n", &mut headers),
            InvalidHeaderFormat,
        );
        fail(
            Request::parse(b"G1ET / HTTP/1.A\r\n\r\n", &mut headers),
            InvalidHeaderFormat,
        );
    }
}
