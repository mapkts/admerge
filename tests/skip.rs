use admerge::*;
use std::io::Cursor;
use std::str;

#[test]
fn test_skip_head_bytes() {
    let mut c1 = Cursor::new("foo bar baz ");
    let mut c2 = Cursor::new("bar baz foo ");
    let mut c3 = Cursor::new("baz foo bar ");
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    merger.skip_head(Skip::Bytes(0));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            "foo bar baz bar baz foo baz foo bar "
        ),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::Bytes(1));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            "oo bar baz ar baz foo az foo bar "
        ),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::Bytes(8));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), "baz foo bar "),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::Bytes(12));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), ""),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::Bytes(13));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert!(false),
        Err(e) => match e {
            ErrorKind::InvalidSkip => assert!(true),
            _ => assert!(false),
        },
    }
}

#[test]
fn test_skip_head_bytes_once() {
    let mut c1 = Cursor::new("foo bar baz ");
    let mut c2 = Cursor::new("bar baz foo ");
    let mut c3 = Cursor::new("baz foo bar ");
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    merger.skip_head(Skip::BytesOnce(0));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            "foo bar baz bar baz foo baz foo bar "
        ),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::BytesOnce(1));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            "foo bar baz ar baz foo az foo bar "
        ),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::BytesOnce(8));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), "foo bar baz foo bar "),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::BytesOnce(12));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), "foo bar baz "),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::BytesOnce(13));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert!(false),
        Err(e) => match e {
            ErrorKind::InvalidSkip => assert!(true),
            _ => assert!(false),
        },
    }
}

#[test]
fn test_skip_head_lines() {
    // Test if merger works correctly when merging sources that end with newline.
    let mut c1 = Cursor::new(" 11\n 12\n 13\n");
    let mut c2 = Cursor::new(" 21\n 22\n 23\n");
    let mut c3 = Cursor::new(" 31\n 32\n 33\n");
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    merger.skip_head(Skip::Lines(0));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            " 11\n 12\n 13\n 21\n 22\n 23\n 31\n 32\n 33\n"
        ),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::Lines(1));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            " 12\n 13\n 22\n 23\n 32\n 33\n"
        ),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::Lines(2));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), " 13\n 23\n 33\n"),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::Lines(3));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), ""),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::Lines(4));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert!(false),
        Err(e) => match e {
            ErrorKind::InvalidSkip => assert!(true),
            _ => assert!(false),
        },
    }

    // Test if merger works correctly when merging sources that end with newline.
    let mut c1 = Cursor::new(" 11\n 12\n 13");
    let mut c2 = Cursor::new(" 21\n 22\n 23");
    let mut c3 = Cursor::new(" 31\n 32\n 33");
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    merger.skip_head(Skip::Lines(0));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            " 11\n 12\n 13 21\n 22\n 23 31\n 32\n 33"
        ),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::Lines(1));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), " 12\n 13 22\n 23 32\n 33"),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::Lines(2));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), " 13 23 33"),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::Lines(3));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), ""),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::Lines(4));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert!(false),
        Err(e) => match e {
            ErrorKind::InvalidSkip => assert!(true),
            _ => assert!(false),
        },
    }
}

#[test]
fn test_skip_head_lines_once() {
    // Test if merger works correctly when merging sources that end with newline.
    let mut c1 = Cursor::new(" 11\n 12\n 13\n");
    let mut c2 = Cursor::new(" 21\n 22\n 23\n");
    let mut c3 = Cursor::new(" 31\n 32\n 33\n");
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    merger.skip_head(Skip::LinesOnce(0));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            " 11\n 12\n 13\n 21\n 22\n 23\n 31\n 32\n 33\n"
        ),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::LinesOnce(1));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            " 11\n 12\n 13\n 22\n 23\n 32\n 33\n"
        ),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::LinesOnce(2));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), " 11\n 12\n 13\n 23\n 33\n"),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::LinesOnce(3));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), " 11\n 12\n 13\n"),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::LinesOnce(4));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert!(false),
        Err(e) => match e {
            ErrorKind::InvalidSkip => assert!(true),
            _ => assert!(false),
        },
    }

    // Test if merger works correctly when merging sources that don't end with newline.
    let mut c1 = Cursor::new(" 11\n 12\n 13");
    let mut c2 = Cursor::new(" 21\n 22\n 23");
    let mut c3 = Cursor::new(" 31\n 32\n 33");
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    merger.skip_head(Skip::LinesOnce(0));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            " 11\n 12\n 13 21\n 22\n 23 31\n 32\n 33"
        ),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::LinesOnce(1));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            " 11\n 12\n 13 22\n 23 32\n 33"
        ),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::LinesOnce(2));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), " 11\n 12\n 13 23 33"),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::LinesOnce(3));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), " 11\n 12\n 13"),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::LinesOnce(4));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert!(false),
        Err(e) => match e {
            ErrorKind::InvalidSkip => assert!(true),
            _ => assert!(false),
        },
    }
}

#[test]
fn test_skip_head_repeats() {
    let mut c1 = Cursor::new("foo foo bar ");
    let mut c2 = Cursor::new("foo foo bar ");
    let mut c3 = Cursor::new("foo bar bar ");
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    merger.skip_head(Skip::Repeats("".as_bytes()));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            "foo foo bar foo foo bar foo bar bar "
        ),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::Repeats("bar ".as_bytes()));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            "foo foo bar foo foo bar foo bar bar "
        ),
        Err(_) => assert!(false),
    }

    merger.skip_head(Skip::Repeats("foo ".as_bytes()));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), "bar bar bar bar "),
        Err(_) => assert!(false),
    }

    let mut c1 = Cursor::new("foo foo foo ");
    let mut c2 = Cursor::new("foo foo bar ");
    let mut c3 = Cursor::new("foo bar bar ");

    merger.skip_head(Skip::Repeats("foo ".as_bytes()));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), "bar bar bar "),
        Err(_) => assert!(false),
    }
}

#[test]
fn test_skip_head_until() {
    let mut c1 = Cursor::new("until");
    let mut c2 = Cursor::new(" skip until");
    let mut c3 = Cursor::new(" skip until untouched ");
    let mut c4 = Cursor::new(" skip ");
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    merger.skip_head(Skip::Until("until".as_bytes()));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3, &mut c4], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), " untouched "),
        Err(_) => assert!(false),
    }
}

#[test]
fn test_skip_head_before() {
    let mut c1 = Cursor::new("before");
    let mut c2 = Cursor::new(" skip before");
    let mut c3 = Cursor::new(" skip before untouched ");
    let mut c4 = Cursor::new(" skip ");
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    merger.skip_head(Skip::Before("before".as_bytes()));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3, &mut c4], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            "beforebeforebefore untouched "
        ),
        Err(_) => assert!(false),
    }
}

#[test]
fn test_skip_tail_bytes() {
    let mut c1 = Cursor::new("foo bar baz ");
    let mut c2 = Cursor::new("bar baz foo ");
    let mut c3 = Cursor::new("baz foo bar ");
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    merger.skip_tail(Skip::Bytes(0));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            "foo bar baz bar baz foo baz foo bar "
        ),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::Bytes(1));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            "foo bar bazbar baz foobaz foo bar"
        ),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::Bytes(8));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), "foo bar baz "),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::Bytes(12));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), ""),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::Bytes(13));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert!(false),
        Err(e) => match e {
            ErrorKind::InvalidSkip => assert!(true),
            _ => assert!(false),
        },
    }
}

#[test]
fn test_skip_tail_bytes_once() {
    let mut c1 = Cursor::new("foo bar baz ");
    let mut c2 = Cursor::new("bar baz foo ");
    let mut c3 = Cursor::new("baz foo bar ");
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    merger.skip_tail(Skip::BytesOnce(0));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            "foo bar baz bar baz foo baz foo bar "
        ),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::BytesOnce(1));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            "foo bar bazbar baz foobaz foo bar "
        ),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::BytesOnce(8));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), "foo bar baz foo bar "),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::BytesOnce(12));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), "baz foo bar "),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::BytesOnce(13));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert!(false),
        Err(e) => match e {
            ErrorKind::InvalidSkip => assert!(true),
            _ => assert!(false),
        },
    }
}

#[test]
fn test_skip_tail_lines() {
    // Test if merger works correctly when merging sources that end with newline.
    let mut c1 = Cursor::new(" 11\n 12\n 13\n");
    let mut c2 = Cursor::new(" 21\n 22\n 23\n");
    let mut c3 = Cursor::new(" 31\n 32\n 33\n");
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    merger.skip_tail(Skip::Lines(0));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            " 11\n 12\n 13\n 21\n 22\n 23\n 31\n 32\n 33\n"
        ),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::Lines(1));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            " 11\n 12\n 21\n 22\n 31\n 32\n"
        ),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::Lines(2));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), " 11\n 21\n 31\n"),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::Lines(3));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), ""),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::Lines(4));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert!(false),
        Err(e) => match e {
            ErrorKind::InvalidSkip => assert!(true),
            _ => assert!(false),
        },
    }

    // Test if merger works correctly when merging sources that end with newline.
    let mut c1 = Cursor::new(" 11\n 12\n 13");
    let mut c2 = Cursor::new(" 21\n 22\n 23");
    let mut c3 = Cursor::new(" 31\n 32\n 33");
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    merger.skip_tail(Skip::Lines(0));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            " 11\n 12\n 13 21\n 22\n 23 31\n 32\n 33"
        ),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::Lines(1));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            " 11\n 12\n 21\n 22\n 31\n 32\n"
        ),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::Lines(2));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), " 11\n 21\n 31\n"),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::Lines(3));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), ""),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::Lines(4));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert!(false),
        Err(e) => match e {
            ErrorKind::InvalidSkip => assert!(true),
            _ => assert!(false),
        },
    }
}

#[test]
fn test_skip_tail_lines_once() {
    // Test if merger works correctly when merging sources that end with newline.
    let mut c1 = Cursor::new(" 11\n 12\n 13\n");
    let mut c2 = Cursor::new(" 21\n 22\n 23\n");
    let mut c3 = Cursor::new(" 31\n 32\n 33\n");
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    merger.skip_tail(Skip::LinesOnce(0));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            " 11\n 12\n 13\n 21\n 22\n 23\n 31\n 32\n 33\n"
        ),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::LinesOnce(1));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            " 11\n 12\n 21\n 22\n 31\n 32\n 33\n"
        ),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::LinesOnce(2));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), " 11\n 21\n 31\n 32\n 33\n"),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::LinesOnce(3));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), " 31\n 32\n 33\n"),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::LinesOnce(4));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert!(false),
        Err(e) => match e {
            ErrorKind::InvalidSkip => assert!(true),
            _ => assert!(false),
        },
    }

    // Test if merger works correctly when merging sources that don't end with newline.
    let mut c1 = Cursor::new(" 11\n 12\n 13");
    let mut c2 = Cursor::new(" 21\n 22\n 23");
    let mut c3 = Cursor::new(" 31\n 32\n 33");
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    merger.skip_tail(Skip::LinesOnce(0));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            " 11\n 12\n 13 21\n 22\n 23 31\n 32\n 33"
        ),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::LinesOnce(1));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            " 11\n 12\n 21\n 22\n 31\n 32\n 33"
        ),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::LinesOnce(2));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), " 11\n 21\n 31\n 32\n 33"),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::LinesOnce(3));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), " 31\n 32\n 33"),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::LinesOnce(4));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert!(false),
        Err(e) => match e {
            ErrorKind::InvalidSkip => assert!(true),
            _ => assert!(false),
        },
    }
}

#[test]
fn test_skip_tail_repeats() {
    let mut c1 = Cursor::new("foo foo bar ");
    let mut c2 = Cursor::new("foo foo bar ");
    let mut c3 = Cursor::new("foo bar bar ");
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    merger.skip_tail(Skip::Repeats("".as_bytes()));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            "foo foo bar foo foo bar foo bar bar "
        ),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::Repeats("foo ".as_bytes()));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            "foo foo bar foo foo bar foo bar bar "
        ),
        Err(_) => assert!(false),
    }

    merger.skip_tail(Skip::Repeats("bar ".as_bytes()));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), "foo foo foo foo foo "),
        Err(e) => {
            println!("{:?}", e);
            assert!(false)
        }
    }

    let mut c1 = Cursor::new("foo foo foo ");
    let mut c2 = Cursor::new("foo foo bar ");
    let mut c3 = Cursor::new("bar bar bar ");

    merger.skip_tail(Skip::Repeats("bar ".as_bytes()));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), "foo foo foo foo foo "),
        Err(_) => assert!(false),
    }
}

#[test]
fn test_skip_tail_until() {
    let mut c1 = Cursor::new("until");
    let mut c2 = Cursor::new("until skip ");
    let mut c3 = Cursor::new("untouched until skip ");
    let mut c4 = Cursor::new(" skip ");
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    merger.skip_tail(Skip::Until("until".as_bytes()));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3, &mut c4], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), "untouched "),
        Err(_) => assert!(false),
    }
}

#[test]
fn test_skip_tail_before() {
    let mut c1 = Cursor::new("before");
    let mut c2 = Cursor::new(" before skip");
    let mut c3 = Cursor::new(" untouched before skip ");
    let mut c4 = Cursor::new(" skip ");
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    merger.skip_tail(Skip::Before("before".as_bytes()));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3, &mut c4], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            "before before untouched before"
        ),
        Err(_) => assert!(false),
    }
}
