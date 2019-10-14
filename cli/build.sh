echo "Build x86_64-unknown-linux-gnu"
cargo build --release --target=x86_64-unknown-linux-gnu

echo "Build x86_64-apple-darwin"
PATH="$HOME/osxcross/target/bin:$PATH"  CC=o64-clang CXX=o64-clang++ cargo build --release --target=x86_64-apple-darwin

echo "Build x86_64-pc-windows-gnu"
cargo build --release --target=x86_64-pc-windows-gnu
