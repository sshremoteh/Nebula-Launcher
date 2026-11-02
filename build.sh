#!/usr/bin/env bash
# ==============================================================================
# Horizon Android - Autonomous Compilation Script
# ==============================================================================

set -euo pipefail

# Script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for terminal output
if [[ -t 1 ]]; then
    BOLD="\033[1m"
    GREEN="\033[1;32m"
    BLUE="\033[1;34m"
    YELLOW="\033[1;33m"
    RED="\033[1;31m"
    CYAN="\033[1;36m"
    RESET="\033[0m"
else
    BOLD=""
    GREEN=""
    BLUE=""
    YELLOW=""
    RED=""
    CYAN=""
    RESET=""
fi

log_info() {
    echo -e "${BLUE}[INFO]${RESET} $*"
}

log_ok() {
    echo -e "${GREEN}[SUCCESS]${RESET} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${RESET} $*"
}

log_err() {
    echo -e "${RED}[ERROR]${RESET} $*" >&2
}

banner() {
    echo -e "${CYAN}${BOLD}"
    echo "  ╔══════════════════════════════════════════════════════════╗"
    echo "  ║              Horizon Android Build System                ║"
    echo "  ║        Minecraft Java Edition 26.3 Rust Client           ║"
    echo "  ╚══════════════════════════════════════════════════════════╝"
    echo -e "${RESET}"
}

usage() {
    banner
    echo "Использование: ./build.sh [КОМАНДА] [ОПЦИИ]"
    echo ""
    echo "Команды:"
    echo "  check         Быстрая проверка компиляции кода (cargo check)"
    echo "  client, host  Сборка клиента Horizon для текущей платформы (profile fast)"
    echo "  release       Сборка оптимизированного релиза клиента (profile release)"
    echo "  clippy        Запуск статического анализатора кода Clippy"
    echo "  apk [ABI]     Сборка Android APK пакета (arm64-v8a, x86_64)"
    echo "  apk-check     Проверка компиляции Android таргета"
    echo "  clean         Очистка временных файлов сборки (target/)"
    echo "  help, -h      Показать данную справку"
    echo ""
    echo "Примеры:"
    echo "  ./build.sh check"
    echo "  ./build.sh client"
    echo "  ./build.sh apk arm64-v8a"
    echo ""
    exit 0
}

setup_environment() {
    log_info "Настройка окружения..."

    # Multiarch pkg-config path setup for Linux/Debian/Ubuntu/Termux
    local pkg_paths=(
        "/usr/lib/aarch64-linux-gnu/pkgconfig"
        "/usr/lib/x86_64-linux-gnu/pkgconfig"
        "/usr/lib/arm-linux-gnueabihf/pkgconfig"
        "/usr/share/pkgconfig"
        "/usr/local/lib/pkgconfig"
    )
    for p in "${pkg_paths[@]}"; do
        if [[ -d "$p" ]]; then
            export PKG_CONFIG_PATH="${p}:${PKG_CONFIG_PATH:-}"
        fi
    done

    # Android SDK setup
    if [[ -z "${ANDROID_HOME:-}" ]]; then
        if [[ -d "/opt/android-sdk" ]]; then
            export ANDROID_HOME="/opt/android-sdk"
        elif [[ -d "$HOME/Android/Sdk" ]]; then
            export ANDROID_HOME="$HOME/Android/Sdk"
        fi
    fi
    if [[ -n "${ANDROID_HOME:-}" ]]; then
        export ANDROID_SDK_ROOT="$ANDROID_HOME"
        if [[ -d "$ANDROID_HOME/build-tools" ]]; then
            export PATH="$ANDROID_HOME/build-tools:$PATH"
        fi
    fi

    # Java setup
    if [[ -z "${JAVA_HOME:-}" ]]; then
        if command -v java >/dev/null 2>&1; then
            local java_bin
            java_bin="$(readlink -f "$(command -v java)" 2>/dev/null || command -v java)"
            local java_dir
            java_dir="$(dirname "$(dirname "$java_bin")")"
            if [[ -d "$java_dir/include" || -d "$java_dir/lib" ]]; then
                export JAVA_HOME="$java_dir"
            fi
        fi
    fi

    # Check Rust / Cargo
    if ! command -v cargo >/dev/null 2>&1; then
        log_err "Cargo не найден в PATH! Установите Rust (https://rustup.rs)."
        exit 1
    fi

    local rust_version
    rust_version="$(rustc --version 2>/dev/null || echo 'Unknown')"
    local cargo_version
    cargo_version="$(cargo --version 2>/dev/null || echo 'Unknown')"
    log_info "Rust: $rust_version"
    log_info "Cargo: $cargo_version"

    if [[ -n "${JAVA_HOME:-}" ]]; then
        log_info "JAVA_HOME: $JAVA_HOME"
    fi
    if [[ -n "${ANDROID_HOME:-}" ]]; then
        log_info "ANDROID_HOME: $ANDROID_HOME"
    fi
}

cmd_check() {
    setup_environment
    log_info "Запуск проверки компиляции (cargo check -p horizon-client --lib)..."
    cargo check -p horizon-client --lib
    log_ok "Проверка кода успешно пройдена!"
}

cmd_client() {
    setup_environment
    log_info "Компиляция horizon-client (профиль fast)..."
    cargo build --profile fast -p horizon-client
    log_ok "Сборка клиента успешно завершена!"
    if [[ -f "target/fast/horizon-client" ]]; then
        log_info "Исполняемый файл: target/fast/horizon-client"
    fi
}

cmd_release() {
    setup_environment
    log_info "Компиляция horizon-client (профиль release)..."
    cargo build --release -p horizon-client
    log_ok "Релизная сборка клиента успешно завершена!"
    if [[ -f "target/release/horizon-client" ]]; then
        log_info "Исполняемый файл: target/release/horizon-client"
    fi
}

cmd_clippy() {
    setup_environment
    log_info "Запуск Clippy linter..."
    cargo clippy -p horizon-client --lib -- -D warnings
    log_ok "Clippy проверка успешно пройдена!"
}

cmd_apk() {
    local profile="${1:-fast}"
    if [[ "$profile" == "arm64-v8a" || -z "$profile" ]]; then
        profile="fast"
    fi
    setup_environment

    if [[ -x "/data/data/com.termux/files/usr/bin/aarch64-linux-android-clang" ]]; then
        export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="/data/data/com.termux/files/usr/bin/aarch64-linux-android-clang"
    fi

    log_info "Компиляция нативной библиотеки horizon-android для aarch64-linux-android (profile: $profile)..."
    cargo build --profile "$profile" -p horizon-android --target aarch64-linux-android
    log_ok "Нативная библиотека собрана!"

    log_info "Упаковка и подпись APK..."
    python3 tools/package_apk.py "$profile"
    log_ok "APK пакет готов!"
}

cmd_apk_check() {
    local abi="${1:-arm64-v8a}"
    setup_environment
    log_info "Проверка компиляции Android таргета ($abi)..."
    python3 tools/android_build.py --abi "$abi" --check || {
        log_warn "Проверка через android_build.py завершилась, выполняю cargo check..."
        cargo check -p horizon-client --lib
    }
    log_ok "Проверка Android таргета завершена!"
}

cmd_clean() {
    log_info "Очистка артефактов сборки..."
    cargo clean
    rm -rf .local/android/apk .local/android/gradle-build
    log_ok "Очистка завершена."
}

main() {
    local cmd="${1:-check}"
    case "$cmd" in
        check)
            cmd_check
            ;;
        client|host)
            cmd_client
            ;;
        release)
            cmd_release
            ;;
        clippy)
            cmd_clippy
            ;;
        apk)
            shift || true
            cmd_apk "${1:-arm64-v8a}"
            ;;
        apk-check)
            shift || true
            cmd_apk_check "${1:-arm64-v8a}"
            ;;
        clean)
            cmd_clean
            ;;
        help|-h|--help)
            usage
            ;;
        all)
            cmd_check
            cmd_client
            ;;
        *)
            log_err "Неизвестная команда: $cmd"
            echo "Используйте './build.sh help' для списка доступных команд."
            exit 1
            ;;
    esac
}

main "$@"
