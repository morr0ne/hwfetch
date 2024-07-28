PREFIX := "usr"
BINARY := PREFIX / "bin"
DESTDIR := "/"


build:
    cargo build --release

install:
    install -Dm0755 target/release/hwfetch {{ DESTDIR }}{{ BINARY }}/hwfetch
