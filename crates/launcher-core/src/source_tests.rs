use crate::error::CoreError;
use crate::source::Source;
use std::io::{Read, Write};
use std::net::TcpListener;

/// Kleiner Testserver: beantwortet `n` Anfragen mit dem gleichen Inhalt.
fn serve(body: Vec<u8>, n: usize) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for _ in 0..n {
            let (mut s, _) = listener.accept().unwrap();
            let mut req = [0u8; 2048];
            let _ = s.read(&mut req);
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            s.write_all(head.as_bytes()).unwrap();
            s.write_all(&body).unwrap();
        }
    });
    port
}

#[test]
fn parse_erlaubt_https_und_lokales_http_und_ordner() {
    assert!(
        matches!(Source::parse("https://example.org/katalog/").unwrap(), Source::Http(b) if b == "https://example.org/katalog")
    );
    assert!(matches!(
        Source::parse("http://127.0.0.1:8080").unwrap(),
        Source::Http(_)
    ));
    assert!(matches!(
        Source::parse("http://localhost:8080/x").unwrap(),
        Source::Http(_)
    ));
    assert!(matches!(
        Source::parse(r"C:\katalog").unwrap(),
        Source::Dir(_)
    ));
}

#[test]
fn parse_lehnt_fremdes_http_und_getarnte_hosts_ab() {
    for bad in [
        "http://example.org/katalog",
        "http://127.0.0.1.evil.example",
        "http://localhost@evil.example",
        "http://localhostevil.example",
    ] {
        assert!(
            matches!(Source::parse(bad), Err(CoreError::SourceNotAllowed(_))),
            "{bad} hätte abgelehnt werden müssen"
        );
    }
}

#[test]
fn ordner_quelle_liest_und_kopiert_mit_fortschritt() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("packages")).unwrap();
    std::fs::write(dir.path().join("catalog.json"), b"{}").unwrap();
    std::fs::write(dir.path().join("packages/a.exe"), vec![1u8; 150_000]).unwrap();
    let src = Source::Dir(dir.path().to_path_buf());
    assert_eq!(src.fetch_bytes("catalog.json").unwrap(), b"{}");

    let out = tempfile::tempdir().unwrap();
    let dest = out.path().join("copy.exe");
    let mut last = (0u64, None);
    src.fetch_to_file("packages/a.exe", &dest, &mut |d, t| last = (d, t))
        .unwrap();
    assert_eq!(std::fs::read(&dest).unwrap().len(), 150_000);
    assert_eq!(last, (150_000, Some(150_000)));
}

#[test]
fn ordner_quelle_verlaesst_den_ordner_nie() {
    let dir = tempfile::tempdir().unwrap();
    let src = Source::Dir(dir.path().to_path_buf());
    let out = tempfile::tempdir().unwrap();
    let dest = out.path().join("x");
    for bad in [
        "../geheim.txt",
        "a/../../b",
        "/etc/passwd",
        r"C:\Windows\win.ini",
        r"..\x",
        "http://example.org/x.exe",
    ] {
        let r = src.fetch_to_file(bad, &dest, &mut |_, _| {});
        assert!(
            matches!(r, Err(CoreError::SourceNotAllowed(_))),
            "{bad} hätte abgelehnt werden müssen"
        );
    }
    assert!(matches!(
        src.fetch_bytes("../x"),
        Err(CoreError::SourceNotAllowed(_))
    ));
}

#[test]
fn http_quelle_laedt_katalog_und_paket() {
    let port = serve(vec![9u8; 200_000], 2);
    let src = Source::parse(&format!("http://127.0.0.1:{port}")).unwrap();
    assert_eq!(src.fetch_bytes("catalog.json").unwrap().len(), 200_000);

    let out = tempfile::tempdir().unwrap();
    let dest = out.path().join("p.exe");
    let mut last = (0u64, None);
    src.fetch_to_file("packages/p.exe", &dest, &mut |d, t| last = (d, t))
        .unwrap();
    assert_eq!(last, (200_000, Some(200_000)));
}

#[test]
fn http_quelle_lehnt_fremde_absolute_adressen_ab() {
    let src = Source::Http("https://example.org/k".into());
    let out = tempfile::tempdir().unwrap();
    let r = src.fetch_to_file(
        "http://evil.example/x.exe",
        &out.path().join("x"),
        &mut |_, _| {},
    );
    assert!(matches!(r, Err(CoreError::SourceNotAllowed(_))));
}
