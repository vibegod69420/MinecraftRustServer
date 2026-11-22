#!/usr/bin/env bash
set -euo pipefail

LOG_FILE="/var/log/mc-rust-bootstrap.log"
DONE_FILE="/var/log/mc-rust-bootstrap.done"

exec > >(tee -a "$LOG_FILE") 2>&1
echo "=== mc-rust bootstrap started at $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="

# Add 4 GiB swapfile so rustc / cargo install / JDK 21 never OOM on 2 GiB t4g.small
if [ ! -f /swapfile ]; then
  fallocate -l 4G /swapfile
  chmod 600 /swapfile
  mkswap /swapfile
  swapon /swapfile
  grep -q '/swapfile' /etc/fstab || echo '/swapfile none swap sw 0 0' >> /etc/fstab
fi

export DEBIAN_FRONTEND=noninteractive
apt-get update -y
apt-get install -y --no-install-recommends \
  build-essential pkg-config libssl-dev \
  openjdk-21-jdk \
  tmux jq python3 python3-pip python3-venv \
  git rsync unzip curl ca-certificates \
  heaptrack linux-tools-common linux-tools-generic linux-tools-aws || true

# Install perf package matching active kernel if available
apt-get install -y "linux-tools-$(uname -r)" || true

# Install Gradle 8.14.8 (apt gradle on Ubuntu is too old for JDK 21)
GRADLE_VER="8.14.8"
if [ ! -d "/opt/gradle/gradle-${GRADLE_VER}" ]; then
  mkdir -p /opt/gradle
  curl -fsSL "https://services.gradle.org/distributions/gradle-${GRADLE_VER}-bin.zip" -o /tmp/gradle.zip
  unzip -q -o /tmp/gradle.zip -d /opt/gradle
  rm -f /tmp/gradle.zip
  ln -sfn "/opt/gradle/gradle-${GRADLE_VER}/bin/gradle" /usr/local/bin/gradle
fi

# Install Vineflower 1.12.0 decompiler
VF_VER="1.12.0"
mkdir -p /opt/tools
if [ ! -f "/opt/tools/vineflower.jar" ]; then
  curl -fsSL "https://github.com/Vineflower/vineflower/releases/download/${VF_VER}/vineflower-${VF_VER}.jar" \
    -o /opt/tools/vineflower.jar
fi
cat > /usr/local/bin/vineflower <<'EOF'
#!/usr/bin/env bash
exec java -jar /opt/tools/vineflower.jar "$@"
EOF
chmod +x /usr/local/bin/vineflower

# Workspace directory
mkdir -p /opt/mc-rust
chown -R ubuntu:ubuntu /opt/mc-rust /opt/tools

# Install Rust toolchain + cargo-mutants + cargo-fuzz + flamegraph as ubuntu user
sudo -u ubuntu -H bash -lc '
set -euo pipefail
if [ ! -x "$HOME/.cargo/bin/rustup" ]; then
  curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
fi
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_BUILD_JOBS=2
rustup toolchain install stable
rustup toolchain install nightly
rustup default stable
cargo install --locked cargo-mutants || cargo install cargo-mutants
cargo install --locked cargo-fuzz || cargo install cargo-fuzz
cargo install --locked flamegraph || cargo install flamegraph
'

cat > /etc/profile.d/mc-rust.sh <<'EOF'
export PATH="/home/ubuntu/.cargo/bin:/opt/gradle/gradle-8.14.8/bin:$PATH"
export JAVA_HOME="/usr/lib/jvm/java-21-openjdk-arm64"
EOF
chmod +x /etc/profile.d/mc-rust.sh

touch "$DONE_FILE"
echo "=== mc-rust bootstrap completed at $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
