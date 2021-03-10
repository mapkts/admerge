use std::io::prelude::*;
use std::path::Path;

use admerge::*;
use tempfile::{tempdir, NamedTempFile};

macro_rules! tempfiles {
    () => {{
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        let mut file3 = NamedTempFile::new().unwrap();
        write!(&mut file1, " 11\n 12\n 13").unwrap();
        write!(&mut file2, " 21\n 22\n 23").unwrap();
        write!(&mut file3, " 31\n 32\n 33").unwrap();
        vec![file1, file2, file3]
    }};
}

#[test]
fn with_files_bacics() {
    let merger = FileMerger::new();

    let mut buf = Vec::new();
    let files = tempfiles!().into_iter().map(|f| f.into_file()).collect();
    assert!(merger.with_files(files, &mut buf).is_ok());
    assert_eq!(&buf, b" 11\n 12\n 13 21\n 22\n 23 31\n 32\n 33");

    // fn `with_files` does not cosume merger.
    let mut buf = Vec::new();
    let files = tempfiles!().into_iter().map(|f| f.into_file()).collect();
    assert!(merger.with_files(files, &mut buf).is_ok());
    assert_eq!(&buf, b" 11\n 12\n 13 21\n 22\n 23 31\n 32\n 33");
}

#[test]
fn with_paths_basics() {
    let merger = FileMerger::new();
    let tempfiles = tempfiles!();
    let paths: Vec<&Path> = tempfiles.iter().map(|f| f.path()).collect();

    let mut buf = Vec::new();
    assert!(merger.with_paths(paths.clone(), &mut buf).is_ok());
    assert_eq!(&buf, b" 11\n 12\n 13 21\n 22\n 23 31\n 32\n 33");

    // fn `with_paths` does not cosume merger.
    let mut buf = Vec::new();
    assert!(merger.with_paths(paths.clone(), &mut buf).is_ok());
    assert_eq!(&buf, b" 11\n 12\n 13 21\n 22\n 23 31\n 32\n 33");
}

#[test]
fn with_paths_throws_if_given_invalid_paths() {
    let merger = FileMerger::new();
    let tempfiles = tempfiles!();
    let tempdir = tempdir().unwrap().into_path();

    let mut buf = Vec::new();
    let mut paths: Vec<&Path> = tempfiles.iter().map(|f| f.path()).collect();
    paths.push(tempdir.as_path());
    match merger.with_paths(paths.clone(), &mut buf) {
        Err(e) => match e {
            ErrorKind::InvalidPath(3) => assert!(true),
            _ => assert!(false),
        },
        _ => assert!(false),
    }
}

#[test]
fn with_paths_lossy_basics() {
    let merger = FileMerger::new();
    let tempfiles = tempfiles!();
    let paths: Vec<&Path> = tempfiles.iter().map(|f| f.path()).collect();

    let mut buf = Vec::new();
    assert!(merger.with_paths_lossy(paths.clone(), &mut buf).is_ok());
    assert_eq!(&buf, b" 11\n 12\n 13 21\n 22\n 23 31\n 32\n 33");

    // fn `with_paths` does not cosume merger.
    let mut buf = Vec::new();
    assert!(merger.with_paths_lossy(paths.clone(), &mut buf).is_ok());
    assert_eq!(&buf, b" 11\n 12\n 13 21\n 22\n 23 31\n 32\n 33");
}

#[test]
fn with_paths_lossy_accepts_invalid_paths() {
    let merger = FileMerger::new();
    let tempfiles = tempfiles!();
    let tempdir = tempdir().unwrap().into_path();

    let mut buf = Vec::new();
    let mut paths: Vec<&Path> = tempfiles.iter().map(|f| f.path()).collect();
    paths.push(tempdir.as_path());
    assert!(merger.with_paths_lossy(paths.clone(), &mut buf).is_ok());
    assert_eq!(&buf, b" 11\n 12\n 13 21\n 22\n 23 31\n 32\n 33");
}

#[test]
fn skip_head_and_skip_tail() {
    let tempfiles = tempfiles!();
    let paths: Vec<&Path> = tempfiles.iter().map(|f| f.path()).collect();
    let mut merger = FileMerger::new();

    merger.skip_head(Skip::Lines(0));
    merger.skip_tail(Skip::Lines(0));
    let mut buf = Vec::new();
    assert!(merger.with_paths(paths.clone(), &mut buf).is_ok());
    assert_eq!(&buf, b" 11\n 12\n 13 21\n 22\n 23 31\n 32\n 33");

    merger.skip_head(Skip::Lines(1));
    merger.skip_tail(Skip::Lines(1));
    let mut buf = Vec::new();
    assert!(merger.with_paths(paths.clone(), &mut buf).is_ok());
    assert_eq!(&buf, b" 12\n 22\n 32\n");

    merger.skip_head(Skip::Lines(2));
    merger.skip_tail(Skip::Lines(1));
    let mut buf = Vec::new();
    assert!(merger.with_paths(paths.clone(), &mut buf).is_ok());
    assert_eq!(&buf, b"");

    merger.skip_head(Skip::Lines(1));
    merger.skip_tail(Skip::Lines(2));
    let mut buf = Vec::new();
    assert!(merger.with_paths(paths.clone(), &mut buf).is_ok());
    assert_eq!(&buf, b"");

    merger.skip_head(Skip::Lines(2));
    merger.skip_tail(Skip::Lines(2));
    let mut buf = Vec::new();
    match merger.with_paths(paths.clone(), &mut buf) {
        Err(e) => match e {
            ErrorKind::InvalidSkip => assert!(true),
            _ => assert!(false),
        },
        _ => assert!(false),
    }

    merger.skip_head(Skip::Lines(3));
    merger.skip_tail(Skip::Lines(1));
    let mut buf = Vec::new();
    match merger.with_paths(paths.clone(), &mut buf) {
        Err(e) => match e {
            ErrorKind::InvalidSkip => assert!(true),
            _ => assert!(false),
        },
        _ => assert!(false),
    }

    merger.skip_head(Skip::Lines(1));
    merger.skip_tail(Skip::Lines(3));
    let mut buf = Vec::new();
    match merger.with_paths(paths.clone(), &mut buf) {
        Err(e) => match e {
            ErrorKind::InvalidSkip => assert!(true),
            _ => assert!(false),
        },
        _ => assert!(false),
    }
}

#[test]
fn options_cooperate_with_each_other() {
    let tempfiles = tempfiles!();
    let paths: Vec<&Path> = tempfiles.iter().map(|f| f.path()).collect();
    let mut merger = FileMerger::new();

    merger.skip_head(Skip::Lines(1));
    merger.pad_with(Pad::Custom(
        Some(b" leading \n"),
        Some(b" inner"),
        Some(b" ending \n"),
    ));
    merger.force_ending_newline(Newline::Lf);
    let mut buf = Vec::new();
    assert!(merger.with_paths(paths.clone(), &mut buf).is_ok());
    assert_eq!(
        &buf,
        b" leading \n 12\n 13\n inner 22\n 23\n inner 32\n 33\n ending \n"
    );
}
