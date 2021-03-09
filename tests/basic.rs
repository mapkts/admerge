use admerge::*;
use std::io::Cursor;
use std::str;

#[test]
fn test_merge_empty() {
    let mut buf: Vec<u8> = Vec::new();
    let merger = RsMerger::new();
    match merger.merge_sources_into(Vec::<Cursor<&[u8]>>::new(), &mut buf) {
        Err(e) => match e {
            ErrorKind::NothingPassed => assert!(true),
            _ => assert!(false),
        },
        _ => assert!(false),
    }
}

#[test]
fn test_merge_default() {
    let mut c1 = Cursor::new("hello from p1\n");
    let mut c2 = Cursor::new("hello from p2\n");
    let mut c3 = Cursor::new("hello from p3\n");
    let mut buf = Vec::new();
    let merger = RsMerger::new();

    // Only pass c1.
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), "hello from p1\n"),
        Err(_) => assert!(false),
    }

    // Pass c1 and c2.
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            "hello from p1\nhello from p2\n"
        ),
        Err(_) => assert!(false),
    }

    // Pass c1, c2 and c3.
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            "hello from p1\nhello from p2\nhello from p3\n"
        ),
        Err(_) => assert!(false),
    }
}

#[test]
fn test_force_ending_newline() {
    let mut c1 = Cursor::new(" line 1 ");
    let mut c2 = Cursor::new(" line 2 ");
    let mut c3 = Cursor::new(" line 3 ");
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    merger.force_ending_newline(Newline::Lf);
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            std::str::from_utf8(&buf).unwrap(),
            " line 1 \n line 2 \n line 3 \n"
        ),
        Err(_) => assert!(false),
    }

    merger.force_ending_newline(Newline::Crlf);
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            std::str::from_utf8(&buf).unwrap(),
            " line 1 \r\n line 2 \r\n line 3 \r\n"
        ),
        Err(_) => assert!(false),
    }
}
