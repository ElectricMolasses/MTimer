cargo build --release
cargo build --release --target x86_64-pc-windows-gnu

# Copy Linux
mkdir -p ../godot/addons/m_timer/bin/linux_64/release/
cp ./target/release/*.so ../godot/addons/m_timer/bin/linux_64/release/

# Copy Windows
mkdir -p ../godot/addons/m_timer/bin/windows_64/release/
cp ./target/x86_64-pc-windows-gnu/release/*.dll ../godot/addons/m_timer/bin/windows_64/release/
