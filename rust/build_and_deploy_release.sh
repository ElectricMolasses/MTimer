cargo build --release
cp -r ./target/release/*.{d,so} ../godot/addons/m_timer/target/release/
