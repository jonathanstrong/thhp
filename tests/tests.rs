extern crate httpparser;

use httpparser::*;

#[cfg(test)]
mod request {
    use ::*;

    macro_rules! good {
        ($buf: expr) => {
            {
                let mut headers = Vec::<HeaderField>::with_capacity(10);
                let r = Request::parse($buf, &mut headers);
                assert!(r.is_ok());
            }
        }
    }

    macro_rules! fail {
        ($buf: expr, $err: ident) => {
            {
                let mut headers = Vec::<HeaderField>::with_capacity(10);
                let r = Request::parse($buf, &mut headers);
                assert!(r.is_err());
                assert_eq!(*r.err().unwrap().kind(), $err);
            }
        }
    }

    macro_rules! invalid_method {
        ($parse: expr) => { fail!($parse, InvalidMethod) }
    }

    macro_rules! invalid_path {
        ($parse: expr) => { fail!($parse, InvalidPath) }
    }

    macro_rules! invalid_version {
        ($parse: expr) => { fail!($parse, InvalidVersion) }
    }

    macro_rules! invalid_field_name {
        ($parse: expr) => { fail!($parse, InvalidFieldName) }
    }

    macro_rules! invalid_field_value {
        ($parse: expr) => { fail!($parse, InvalidFieldValue) }
    }

    macro_rules! invalid_new_line {
        ($parse: expr) => { fail!($parse, InvalidNewLine) }
    }

    macro_rules! incomplete {
        ($buf: expr) => {
            {
                let mut headers = Vec::<HeaderField>::with_capacity(10);
                let r = Request::parse($buf, &mut headers);
                assert!(r.is_ok());
                assert!(r.unwrap().is_incomplete());
            }
        }
    }

    #[test]
    fn good_request() {
        good!(b"GET / HTTP/1.1\r\n\r\n");
        good!(b"GET / HTTP/1.1\n\n");
        good!(b"GET / HTTP/1.1\r\n\n");
        good!(b"GET / HTTP/1.1\n\r\n");
        good!(b"GET / HTTP/1.1\r\na:b\r\n\r\n");
        good!(b"GET / HTTP/1.1\r\na:b\r\n\n");
        good!(b"GET / HTTP/1.1\r\na:b\n\n");
        good!(b"GET / HTTP/1.1\r\na:b\n\r\n");
    }

    #[test]
    fn bad_request() {
        invalid_method!(b"G\x01ET / HTTP/1.1\r\n\r\n");
        invalid_path!(b"GET /a\x01ef HTTP/1.1\r\n\r\n");
        invalid_version!(b"GET / HOGE\r\n\r\n");
        invalid_version!(b"GET / HTTP/11.1\r\n\r\n");
        invalid_version!(b"GET / HTTP/A.1\r\n\r\n");
        invalid_version!(b"GET / HTTP/1.A\r\n\r\n");
        invalid_version!(b"GET / HTTP/1.A\r\n\r\n");
        invalid_field_name!(b"GET / HTTP/1.1\r\na\x01b:xyz\r\n\r\n");
        invalid_field_value!(b"GET / HTTP/1.1\r\nabc:x\x01z\r\n\r\n");
        invalid_new_line!(b"GET / HTTP/1.1\r\nabc:xyz\ra\n\r\n");
        invalid_new_line!(b"GET / HTTP/1.1\r\nabc:xyz\r\n\ra\n");
    }

    #[test]
    fn incomplete_request() {
        incomplete!(b"");
        incomplete!(b"GET");
        incomplete!(b"GET ");
        incomplete!(b"GET /");
        incomplete!(b"GET / ");
        incomplete!(b"GET / HTT");
        incomplete!(b"GET / HTTP/1.");
        incomplete!(b"GET / HTTP/1.1");
        incomplete!(b"GET / HTTP/1.1\r");
        incomplete!(b"GET / HTTP/1.1\r\n");
        incomplete!(b"GET / HTTP/1.1\r\na");
        incomplete!(b"GET / HTTP/1.1\r\na:");
        incomplete!(b"GET / HTTP/1.1\r\na:b");
        incomplete!(b"GET / HTTP/1.1\r\na:b\r");
        incomplete!(b"GET / HTTP/1.1\r\na:b\r\n");
        incomplete!(b"GET / HTTP/1.1\r\na:b\r\n\r");
    }
}

#[cfg(test)]
mod response {
    use ::*;

    macro_rules! good {
        ($buf: expr) => {
            {
                let mut headers = Vec::<HeaderField>::with_capacity(10);
                let r = Response::parse($buf, &mut headers);
                assert!(r.is_ok());
            }
        }
    }

    macro_rules! fail {
        ($buf: expr, $err: ident) => {
            {
                let mut headers = Vec::<HeaderField>::with_capacity(10);
                let r = Response::parse($buf, &mut headers);
                assert!(r.is_err());
                assert_eq!(*r.err().unwrap().kind(), $err);
            }
        }
    }

    macro_rules! invalid_version {
        ($parse: expr) => { fail!($parse, InvalidVersion) }
    }

    macro_rules! invalid_status_code {
        ($parse: expr) => { fail!($parse, InvalidStatusCode) }
    }

    macro_rules! invalid_reason_phrase {
        ($parse: expr) => { fail!($parse, InvalidReasonPhrase) }
    }

    macro_rules! incomplete {
        ($buf: expr) => {
            {
                let mut headers = Vec::<HeaderField>::with_capacity(10);
                let r = Response::parse($buf, &mut headers);
                assert!(r.is_ok());
                assert!(r.unwrap().is_incomplete());
            }
        }
    }

    #[test]
    fn good_request() {
        good!(b"HTTP/1.1 200 OK\r\n\r\n");
        good!(b"HTTP/1.1 200 OK\n\n");
        good!(b"HTTP/1.1 200 OK\r\n\n");
        good!(b"HTTP/1.1 200 OK\n\r\n");
        good!(b"HTTP/1.1 200 OK\r\na:b\r\n\r\n");
        good!(b"HTTP/1.1 200 OK\r\na:b\r\n\n");
        good!(b"HTTP/1.1 200 OK\r\na:b\n\n");
        good!(b"HTTP/1.1 200 OK\r\na:b\n\r\n");
    }

    #[test]
    fn bad_request() {
        invalid_version!(b"HOGE/1.1 200 OK\r\n\r\n");
        invalid_version!(b"HTTP/11.1 200 OK\r\n\r\n");
        invalid_version!(b"HTTP/A.1 200 OK\r\n\r\n");
        invalid_version!(b"HTTP/1.A 200 OK\r\n\r\n");
        invalid_version!(b"HTTP/1.11 200 OK\r\n\r\n");
        invalid_status_code!(b"HTTP/1.1 20 OK\r\na:b\r\n\r\n");
        invalid_status_code!(b"HTTP/1.1 2000 OK\r\na:b\r\n\r\n");
        invalid_status_code!(b"HTTP/1.1 2A00 OK\r\na:b\r\n\r\n");
        invalid_reason_phrase!(b"HTTP/1.1 200 O\x01K\r\na:b\r\n\r\n");
        invalid_reason_phrase!(b"HTTP/1.1 200 O\x01K\r\na:b\r\n\r\n");
        invalid_reason_phrase!(b"HTTP/1.1 200 O\x01K\r\na\x01:b\r\n\r\n");
    }

    #[test]
    fn incomplete_request() {
        incomplete!(b"");
        incomplete!(b"HTT");
        incomplete!(b"HTTP/");
        incomplete!(b"HTTP/1");
        incomplete!(b"HTTP/1.1");
        incomplete!(b"HTTP/1.1 ");
        incomplete!(b"HTTP/1.1 2");
        incomplete!(b"HTTP/1.1 200");
        incomplete!(b"HTTP/1.1 200 ");
        incomplete!(b"HTTP/1.1 200 O");
        incomplete!(b"HTTP/1.1 200 OK");
        incomplete!(b"HTTP/1.1 200 OK\r");
        incomplete!(b"HTTP/1.1 200 OK\r\n");
        incomplete!(b"HTTP/1.1 200 OK\r\na");
        incomplete!(b"HTTP/1.1 200 OK\r\na:");
        incomplete!(b"HTTP/1.1 200 OK\r\na:b");
        incomplete!(b"HTTP/1.1 200 OK\r\na:b\r");
        incomplete!(b"HTTP/1.1 200 OK\r\na:b\r\n");
        incomplete!(b"HTTP/1.1 200 OK\r\na:b\r\n\r");
    }
}