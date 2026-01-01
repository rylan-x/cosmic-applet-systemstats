# Build the release version
build-release:
    cargo build --release

# Install the applet (requires sudo, run 'just build-release' first)
install:
    install -Dm755 target/release/cosmic-applet-systemstats /usr/bin/cosmic-applet-systemstats
    install -Dm644 res/com.github.rylan-x.systemstats.desktop /usr/share/applications/com.github.rylan-x.systemstats.desktop

# Clean build artifacts
clean:
    cargo clean
