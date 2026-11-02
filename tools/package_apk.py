#!/usr/bin/env python3
"""Build and package a 100% clean, native Horizon Android APK.
Uses official Android SDK tools (aapt2, javac, d8, zipalign, apksigner)
and the native ARM64 Rust library from this repository.
"""

import os
import sys
import shutil
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def run(cmd, **kwargs):
    print(" ".join(map(str, cmd)), flush=True)
    subprocess.run(list(map(str, cmd)), check=True, **kwargs)

def main():
    profile = sys.argv[1] if len(sys.argv) > 1 else "fast"
    lib_path = ROOT / f"target/aarch64-linux-android/{profile}/libhorizon_android.so"
    if not lib_path.is_file():
        for p in ["fast", "release", "dev", "debug"]:
            cand = ROOT / f"target/aarch64-linux-android/{p}/libhorizon_android.so"
            if cand.is_file():
                lib_path = cand
                break

    if not lib_path.is_file():
        print(f"[ERROR] Native library not found at: {lib_path}", file=sys.stderr)
        sys.exit(1)

    print(f"[INFO] Using native library: {lib_path}")

    # Build directories
    work_dir = ROOT / ".local/android/build_clean"
    work_dir.mkdir(parents=True, exist_ok=True)
    res_dir = work_dir / "res"
    classes_dir = work_dir / "classes"
    dex_dir = work_dir / "dex"
    gen_dir = work_dir / "gen"
    out_dir = ROOT / ".local/android/apk"
    out_dir.mkdir(parents=True, exist_ok=True)

    for d in [classes_dir, dex_dir, gen_dir]:
        shutil.rmtree(d, ignore_errors=True)
        d.mkdir(parents=True, exist_ok=True)

    # 1. Compile Android resources
    print("[1/7] Compiling Android resources with aapt2...")
    res_zip = work_dir / "res.zip"
    run(["aapt2", "compile", "--dir", str(ROOT / "packaging/android/res"), "-o", str(res_zip)])

    # 2. Process AndroidManifest.xml
    print("[2/7] Preparing AndroidManifest.xml...")
    manifest_src = (ROOT / "packaging/android/AndroidManifest.xml").read_text(encoding="utf-8")
    manifest_text = manifest_src.replace(
        '<manifest ',
        '<manifest package="dev.horizon.android" android:versionCode="100" android:versionName="1.0" '
    ).replace('${applicationId}', 'dev.horizon.android')
    manifest_file = work_dir / "AndroidManifest.xml"
    manifest_file.write_text(manifest_text, encoding="utf-8")

    # 3. Link APK base resources
    print("[3/7] Linking APK with aapt2...")
    android_jar = Path("/opt/android-sdk/platforms/android-34/android.jar")
    base_apk = work_dir / "base_linked.apk"
    if base_apk.is_file():
        base_apk.unlink()
    run([
        "aapt2", "link",
        "-I", str(android_jar),
        "--min-sdk-version", "28",
        "--target-sdk-version", "34",
        "--manifest", str(manifest_file),
        str(res_zip),
        "--java", str(gen_dir),
        "-o", str(base_apk)
    ])

    # 4. Compile Java classes and DEX
    print("[4/7] Compiling Java classes and DEX with d8...")
    java_files = list((ROOT / "packaging/android/java/dev/horizon/android").glob("*.java"))
    run([
        "javac",
        "-cp", str(android_jar),
        "--release", "17",
        "-d", str(classes_dir),
        *[str(f) for f in java_files]
    ])
    class_files = list(classes_dir.rglob("*.class"))
    run([
        "d8",
        "--min-api", "28",
        "--output", str(dex_dir),
        *[str(f) for f in class_files]
    ])
    dex_file = dex_dir / "classes.dex"
    if not dex_file.is_file():
        raise FileNotFoundError(f"DEX file not generated: {dex_file}")

    # 5. Prepare Assets
    print("[5/7] Preparing engine and control assets...")
    engine_data_zip = ROOT / ".local/android/engine-data.zip"
    if not engine_data_zip.is_file():
        sys.path.insert(0, str(ROOT / "tools"))
        import android_build
        engine_data_zip = android_build.engine_data()

    controls_zip = ROOT / ".local/android/bedrock-controls.zip"

    # 6. Assembly APK
    print("[6/7] Assembling unaligned APK...")
    temp_apk = work_dir / "unaligned.apk"
    if temp_apk.is_file():
        temp_apk.unlink()

    # Strip library
    stripped_lib = work_dir / "libhorizon_android.so"
    shutil.copyfile(lib_path, stripped_lib)
    try:
        run(["strip", "--strip-unneeded", str(stripped_lib)])
    except Exception as e:
        print(f"[WARN] Strip failed: {e}")

    lib_data = stripped_lib.read_bytes()
    dex_data = dex_file.read_bytes()

    with zipfile.ZipFile(base_apk, 'r') as zin, zipfile.ZipFile(temp_apk, 'w') as zout:
        # Copy base resources (AndroidManifest.xml, resources.arsc, res/*)
        for item in zin.infolist():
            zout.writestr(item, zin.read(item.filename))

        # Add classes.dex
        dex_info = zipfile.ZipInfo("classes.dex", (2026, 1, 1, 0, 0, 0))
        dex_info.compress_type = zipfile.ZIP_DEFLATED
        dex_info.external_attr = 0o100644 << 16
        zout.writestr(dex_info, dex_data)

        # Add native library (uncompressed for fast memory mapping)
        for so_name in ["libhorizon_android.so", "libhorizon_client.so"]:
            so_info = zipfile.ZipInfo(f"lib/arm64-v8a/{so_name}", (2026, 1, 1, 0, 0, 0))
            so_info.compress_type = zipfile.ZIP_STORED
            so_info.external_attr = 0o100755 << 16
            zout.writestr(so_info, lib_data)

        # Add Assets
        if engine_data_zip.is_file():
            asset_info = zipfile.ZipInfo("assets/engine-data.zip", (2026, 1, 1, 0, 0, 0))
            asset_info.compress_type = zipfile.ZIP_DEFLATED
            asset_info.external_attr = 0o100644 << 16
            zout.writestr(asset_info, engine_data_zip.read_bytes())

        if controls_zip.is_file():
            asset_info = zipfile.ZipInfo("assets/bedrock-controls.zip", (2026, 1, 1, 0, 0, 0))
            asset_info.compress_type = zipfile.ZIP_DEFLATED
            asset_info.external_attr = 0o100644 << 16
            zout.writestr(asset_info, controls_zip.read_bytes())

        for doc in ["LICENSE", "THIRD_PARTY_NOTICES.md"]:
            doc_p = ROOT / doc
            if doc_p.is_file():
                doc_info = zipfile.ZipInfo(f"assets/{doc}.txt" if doc == "LICENSE" else f"assets/{doc}", (2026, 1, 1, 0, 0, 0))
                doc_info.compress_type = zipfile.ZIP_DEFLATED
                doc_info.external_attr = 0o100644 << 16
                zout.writestr(doc_info, doc_p.read_bytes())

    # 7. Zipalign & Sign
    print("[7/7] Aligning and signing APK...")
    aligned_apk = work_dir / "aligned.apk"
    if aligned_apk.is_file():
        aligned_apk.unlink()
    run(["zipalign", "-f", "-p", "4", str(temp_apk), str(aligned_apk)])

    final_apk = out_dir / "nebula-launcher.apk"
    if final_apk.is_file():
        final_apk.unlink()

    keystore = Path("/opt/android-sdk/debug.keystore")
    run([
        "apksigner", "sign",
        "--ks", str(keystore),
        "--ks-pass", "pass:android",
        "--key-pass", "pass:android",
        "--out", str(final_apk),
        str(aligned_apk)
    ])

    print("[INFO] Verifying APK signature...")
    run(["apksigner", "verify", "--verbose", str(final_apk)])

    sdcard_apk = Path("/sdcard/nebula-launcher.apk")
    sdcard_legacy = Path("/sdcard/horizon-android.apk")
    try:
        shutil.copyfile(final_apk, sdcard_apk)
        shutil.copyfile(final_apk, sdcard_legacy)
        print(f"[SUCCESS] Copied to {sdcard_apk} and {sdcard_legacy}")
    except Exception as e:
        print(f"[WARN] Could not copy to /sdcard: {e}")

    size_mb = final_apk.stat().st_size / (1024 * 1024)
    print(f"\n========================================================")
    print(f"  APK NEBULA LAUNCHER 1.0 RELEASE УСПЕШНО СОБРАН И ПОДПИСАН!")
    print(f"  Приложение: Nebula Launcher")
    print(f"  Версия: 1.0 Release (code 100)")
    print(f"  Путь: {final_apk}")
    print(f"  Внутренняя память телефона: {sdcard_apk}")
    print(f"  Размер: {size_mb:.2f} MiB")
    print(f"========================================================\n")

if __name__ == "__main__":
    main()
