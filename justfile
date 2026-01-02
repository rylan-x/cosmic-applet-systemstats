# Build the release version
build-release:
    cargo build --release

# Install the applet (requires sudo, run 'just build-release' first)
install:
    install -Dm755 target/release/cosmic-applet-systemstats /usr/bin/cosmic-applet-systemstats
    install -Dm644 res/io.github.rylan_x.cosmic-applet-systemstats.desktop /usr/share/applications/io.github.rylan_x.cosmic-applet-systemstats.desktop
    install -Dm644 res/systemstats-icon.svg /usr/share/icons/hicolor/scalable/apps/systemstats-icon.svg

# Clean build artifacts
clean:
    cargo clean
