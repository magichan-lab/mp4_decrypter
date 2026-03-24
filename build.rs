use std::{env, path::PathBuf};

/// FFmpeg 配置解決結果
///
/// @property include_dir FFmpeg ヘッダディレクトリ
/// @property lib_dir FFmpeg ライブラリディレクトリ
struct FfmpegLayout {
    include_dir: PathBuf,
    lib_dir: PathBuf,
}

/// FFmpeg 配置解決処理
///
/// @param root リポジトリルートパス
/// @return 解決済み FFmpeg 配置情報
fn resolve_ffmpeg_layout(root: &PathBuf) -> FfmpegLayout {
    let base_dir = env::var_os("FFMPEG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("third_party").join("ffmpeg"));

    FfmpegLayout { include_dir: base_dir.join("include"), lib_dir: base_dir.join("lib") }
}

/// ビルドスクリプト本体
fn main() {
    let root =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"));
    let ffmpeg = resolve_ffmpeg_layout(&root);
    let shim = root.join("src").join("ffmpeg_shim.c");

    println!("cargo:rerun-if-env-changed=FFMPEG_DIR");
    println!("cargo:rerun-if-changed={}", ffmpeg.include_dir.display());
    println!("cargo:rerun-if-changed={}", ffmpeg.lib_dir.display());
    println!("cargo:rerun-if-changed={}", shim.display());

    if !ffmpeg.include_dir.exists() {
        panic!(
            "FFmpeg include directory not found: {} (set FFMPEG_DIR or place FFmpeg under third_party/ffmpeg)",
            ffmpeg.include_dir.display()
        );
    }
    if !ffmpeg.lib_dir.exists() {
        panic!(
            "FFmpeg lib directory not found: {} (set FFMPEG_DIR or place FFmpeg under third_party/ffmpeg)",
            ffmpeg.lib_dir.display()
        );
    }
    if !shim.exists() {
        panic!("C shim file not found: {}", shim.display());
    }

    println!("cargo:rustc-link-search=native={}", ffmpeg.lib_dir.display());

    println!("cargo:rustc-link-lib=avformat");
    println!("cargo:rustc-link-lib=avcodec");
    println!("cargo:rustc-link-lib=avutil");
    println!("cargo:rustc-link-lib=swresample");
    println!("cargo:rustc-link-lib=swscale");

    println!("cargo:rustc-link-lib=ws2_32");
    println!("cargo:rustc-link-lib=secur32");
    println!("cargo:rustc-link-lib=bcrypt");
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=ole32");
    println!("cargo:rustc-link-lib=uuid");
    println!("cargo:rustc-link-lib=strmiids");

    cc::Build::new().file(shim).include(ffmpeg.include_dir).compile("ffmpeg_shim");
}
