cargo build
cargo build --target x86_64-pc-windows-gnu

# Copy Linux
mkdir -p ../godot/addons/m_timer/bin/linux_64/debug/
cp ./target/debug/*.so ../godot/addons/m_timer/bin/linux_64/debug/

# Copy Windows
mkdir -p ../godot/addons/m_timer/bin/windows_64/debug/
cp ./target/x86_64-pc-windows-gnu/debug/*.dll ../godot/addons/m_timer/bin/windows_64/debug/
