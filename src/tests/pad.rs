use admerge::*;
use std::io::Cursor;
use std::str;

#[test]
fn test_pad_with_before() {
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    let mut c1 = Cursor::new(" c1 ");
    let mut c2 = Cursor::new(" c2 ");
    let mut c3 = Cursor::new(" c3 ");

    merger.pad_with(Pad::Before(b""));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), " c1  c2  c3 "),
        Err(_) => assert!(false),
    }

    merger.pad_with(Pad::Before(b"leading"));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), "leading c1  c2  c3 "),
        Err(_) => assert!(false),
    }
}

#[test]
fn test_pad_with_between() {
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    let mut c1 = Cursor::new(" c1 ");
    let mut c2 = Cursor::new(" c2 ");
    let mut c3 = Cursor::new(" c3 ");

    merger.pad_with(Pad::Between(b""));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), " c1  c2  c3 "),
        Err(_) => assert!(false),
    }

    merger.pad_with(Pad::Between(b"inner"));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), " c1 inner c2 inner c3 "),
        Err(_) => assert!(false),
    }
}

#[test]
fn test_pad_with_after() {
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    let mut c1 = Cursor::new(" c1 ");
    let mut c2 = Cursor::new(" c2 ");
    let mut c3 = Cursor::new(" c3 ");

    merger.pad_with(Pad::After(b""));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), " c1  c2  c3 "),
        Err(_) => assert!(false),
    }

    merger.pad_with(Pad::After(b"ending"));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), " c1  c2  c3 ending"),
        Err(_) => assert!(false),
    }
}

#[test]
fn test_pad_with_custom() {
    let mut buf = Vec::new();
    let mut merger = RsMerger::new();

    let mut c1 = Cursor::new(" c1 ");
    let mut c2 = Cursor::new(" c2 ");
    let mut c3 = Cursor::new(" c3 ");

    merger.pad_with(Pad::Custom(Some(b""), Some(b""), Some(b"")));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(str::from_utf8(&buf).unwrap(), " c1  c2  c3 "),
        Err(_) => assert!(false),
    }

    merger.pad_with(Pad::Custom(
        Some(b"leading"),
        Some(b"inner"),
        Some(b"ending"),
    ));
    buf.clear();
    match merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf) {
        Ok(_) => assert_eq!(
            str::from_utf8(&buf).unwrap(),
            "leading c1 inner c2 inner c3 ending"
        ),
        Err(_) => assert!(false),
    }
}
