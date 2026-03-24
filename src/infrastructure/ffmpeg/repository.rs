//! FFmpeg/File IO 利用 MP4 処理リポジトリ実装

use std::ffi::{CStr, CString};
use std::fs;
use std::path::{Path, PathBuf};
use std::ptr;
use std::time::Duration;

use crate::application::ports::Mp4ProcessingPort;
use crate::domain::entities::{DecryptionProgress, FileEncryptionState};
use crate::domain::errors::AppError;
use crate::domain::services::OutputNamingService;
use crate::domain::value_objects::DecryptionKey;
use crate::infrastructure::ffmpeg::ffi::*;

/// FFmpeg ベース MP4 処理リポジトリ
#[derive(Debug, Default, Clone, Copy)]
pub struct FfmpegMp4ProcessingRepository;

/// FFmpeg 入力コンテキスト RAII ラッパー
///
/// @property 0 `AVFormatContext` ポインタ
struct InputContext(*mut AVFormatContext);

impl Drop for InputContext {
    /// 入力コンテキスト自動解放処理
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_null() {
                avformat_close_input(&mut self.0);
            }
        }
    }
}

/// FFmpeg 出力コンテキスト RAII ラッパー
///
/// @property 0 `AVFormatContext` ポインタ
struct OutputContext(*mut AVFormatContext);

impl Drop for OutputContext {
    /// 出力コンテキストおよび IO 自動解放処理
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_null() {
                let pb = jk_avformat_pb(self.0);
                if !pb.is_null() {
                    let mut pb_mut = pb;
                    let _ = avio_closep(&mut pb_mut);
                    jk_avformat_set_pb(self.0, ptr::null_mut());
                }
                avformat_free_context(self.0);
                self.0 = ptr::null_mut();
            }
        }
    }
}

/// FFmpeg パケット RAII ラッパー
///
/// @property 0 `AVPacket` ポインタ
struct Packet(*mut AVPacket);

impl Drop for Packet {
    /// パケット自動解放処理
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_null() {
                jk_av_packet_free(&mut self.0);
            }
        }
    }
}

impl FfmpegMp4ProcessingRepository {
    /// FFmpeg エラーコード可読メッセージ変換処理
    ///
    /// @param ret FFmpeg 関数戻り値
    /// @return 可読化済みエラーメッセージ
    fn ff_error(ret: i32) -> String {
        unsafe {
            let mut buffer = [0i8; 256];
            let _ = av_strerror(ret, buffer.as_mut_ptr(), buffer.len());
            CStr::from_ptr(buffer.as_ptr()).to_string_lossy().into_owned()
        }
    }

    /// インフラエラー生成処理
    ///
    /// @param message エラーメッセージ
    /// @return インフラ層エラー
    fn infra_error(message: impl Into<String>) -> AppError {
        AppError::Infrastructure(message.into())
    }

    /// 一時停止要求待機処理
    ///
    /// @param is_cancelled キャンセル要求判定クロージャ
    /// @param is_paused 一時停止要求判定クロージャ
    /// @return 継続可否判定結果
    fn wait_if_paused<F, G>(is_cancelled: &F, is_paused: &G) -> Result<(), AppError>
    where
        F: Fn() -> bool,
        G: Fn() -> bool,
    {
        while is_paused() {
            if is_cancelled() {
                return Err(AppError::Cancelled);
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        Ok(())
    }
}

impl Mp4ProcessingPort for FfmpegMp4ProcessingRepository {
    /// MP4 内暗号化関連ボックス簡易判定処理
    ///
    /// @param path 判定対象ファイルパス
    /// @return 暗号化状態またはファイル処理エラー
    fn inspect_encryption(&self, path: &Path) -> Result<FileEncryptionState, AppError> {
        if !path.exists() {
            return Err(AppError::FileSystem(format!(
                "入力ファイルが存在しません: {}",
                path.display()
            )));
        }

        let data = fs::read(path).map_err(|_| {
            AppError::FileSystem(format!("ファイルを読み取れません: {}", path.display()))
        })?;
        let markers = [
            b"encv".as_slice(),
            b"enca".as_slice(),
            b"sinf".as_slice(),
            b"schm".as_slice(),
            b"tenc".as_slice(),
        ];

        if markers.iter().any(|marker| data.windows(marker.len()).any(|window| window == *marker)) {
            Ok(FileEncryptionState::Encrypted)
        } else {
            Ok(FileEncryptionState::Plain)
        }
    }

    /// 出力ファイルパス生成処理
    ///
    /// @param input 入力ファイルパス
    /// @return 復号出力ファイルパス
    fn output_path(&self, input: &Path) -> PathBuf {
        OutputNamingService::build_output_path(input)
    }

    /// FFmpeg remux による復号済み MP4 生成処理
    ///
    /// @param input_path 入力ファイルパス
    /// @param key 復号キー
    /// @param on_progress 進捗通知コールバック
    /// @param is_cancelled キャンセル要求判定クロージャ
    /// @param is_paused 一時停止要求判定クロージャ
    /// @return 生成済み出力ファイルパスまたは復号処理エラー
    fn decrypt<F, C, P>(
        &self,
        input_path: &Path,
        key: &DecryptionKey,
        mut on_progress: F,
        is_cancelled: C,
        is_paused: P,
    ) -> Result<PathBuf, AppError>
    where
        F: FnMut(DecryptionProgress),
        C: Fn() -> bool,
        P: Fn() -> bool,
    {
        if !input_path.exists() {
            return Err(AppError::FileSystem(format!(
                "入力ファイルが存在しません: {}",
                input_path.display()
            )));
        }

        let input_str = input_path.to_string_lossy().to_string();
        let output = self.output_path(input_path);
        let output_str = output.to_string_lossy().to_string();
        let filename = input_path
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| input_str.clone());

        let input_c = CString::new(input_str)
            .map_err(|_| AppError::Validation("入力パスにNUL文字が含まれています".to_string()))?;
        let output_c = CString::new(output_str)
            .map_err(|_| AppError::Validation("出力パスにNUL文字が含まれています".to_string()))?;
        let key_name =
            CString::new("decryption_key").expect("static literal must be valid CString");
        let key_value = CString::new(key.as_str())
            .map_err(|_| AppError::Validation("キーにNUL文字が含まれています".to_string()))?;

        unsafe {
            let mut dict: *mut AVDictionary = ptr::null_mut();
            let mut input_context = InputContext(ptr::null_mut());
            let mut output_context = OutputContext(ptr::null_mut());

            let dict_ret = av_dict_set(&mut dict, key_name.as_ptr(), key_value.as_ptr(), 0);
            if dict_ret < 0 {
                return Err(Self::infra_error(format!(
                    "av_dict_set failed: {}",
                    Self::ff_error(dict_ret)
                )));
            }

            let open_ret = avformat_open_input(
                &mut input_context.0,
                input_c.as_ptr(),
                ptr::null_mut(),
                &mut dict,
            );
            av_dict_free(&mut dict);
            if open_ret < 0 {
                return Err(Self::infra_error(format!(
                    "avformat_open_input failed: {}",
                    Self::ff_error(open_ret)
                )));
            }

            let stream_info_ret = avformat_find_stream_info(input_context.0, ptr::null_mut());
            if stream_info_ret < 0 {
                return Err(Self::infra_error(format!(
                    "avformat_find_stream_info failed: {}",
                    Self::ff_error(stream_info_ret)
                )));
            }

            let alloc_ret = avformat_alloc_output_context2(
                &mut output_context.0,
                ptr::null_mut(),
                ptr::null(),
                output_c.as_ptr(),
            );
            if alloc_ret < 0 || output_context.0.is_null() {
                return Err(Self::infra_error(format!(
                    "avformat_alloc_output_context2 failed: {}",
                    Self::ff_error(alloc_ret)
                )));
            }

            let stream_count = jk_avformat_nb_streams(input_context.0);
            for index in 0..stream_count {
                let in_stream = jk_avformat_stream(input_context.0, index);
                if in_stream.is_null() {
                    return Err(Self::infra_error(format!(
                        "入力ストリーム取得に失敗しました: {}",
                        index
                    )));
                }

                let out_stream = avformat_new_stream(output_context.0, ptr::null());
                if out_stream.is_null() {
                    return Err(Self::infra_error(format!(
                        "出力ストリーム作成に失敗しました: {}",
                        index
                    )));
                }

                let copy_ret = avcodec_parameters_copy(
                    jk_avstream_codecpar(out_stream),
                    jk_avstream_codecpar(in_stream),
                );
                if copy_ret < 0 {
                    return Err(Self::infra_error(format!(
                        "avcodec_parameters_copy failed: {}",
                        Self::ff_error(copy_ret)
                    )));
                }

                jk_avstream_set_time_base(out_stream, jk_avstream_time_base(in_stream));
                jk_avcodecpar_set_codec_tag_zero(jk_avstream_codecpar(out_stream));
            }

            if (jk_avformat_oformat_flags(output_context.0) & AVFMT_NOFILE) == 0 {
                let mut out_pb: *mut AVIOContext = ptr::null_mut();
                let avio_ret = avio_open(&mut out_pb, output_c.as_ptr(), AVIO_FLAG_WRITE);
                if avio_ret < 0 {
                    return Err(Self::infra_error(format!(
                        "avio_open failed: {}",
                        Self::ff_error(avio_ret)
                    )));
                }
                jk_avformat_set_pb(output_context.0, out_pb);
            }

            let header_ret = avformat_write_header(output_context.0, ptr::null_mut());
            if header_ret < 0 {
                return Err(Self::infra_error(format!(
                    "avformat_write_header failed: {}",
                    Self::ff_error(header_ret)
                )));
            }

            let in_pb = jk_avformat_pb(input_context.0);
            let input_size = if in_pb.is_null() { -1 } else { avio_size(in_pb) };
            let mut last_reported_bucket = -1i32;
            let packet = Packet(jk_av_packet_alloc());
            if packet.0.is_null() {
                return Err(Self::infra_error("av_packet_alloc failed"));
            }

            loop {
                if is_cancelled() {
                    return Err(AppError::Cancelled);
                }

                Self::wait_if_paused(&is_cancelled, &is_paused)?;

                let read_ret = av_read_frame(input_context.0, packet.0);
                if read_ret == jk_averror_eof() {
                    break;
                }
                if read_ret < 0 {
                    return Err(Self::infra_error(format!(
                        "av_read_frame failed: {}",
                        Self::ff_error(read_ret)
                    )));
                }

                let stream_index = jk_av_packet_stream_index(packet.0);
                if stream_index < 0 {
                    av_packet_unref(packet.0);
                    return Err(Self::infra_error("packet stream_index is invalid"));
                }

                let in_stream = jk_avformat_stream(input_context.0, stream_index as u32);
                let out_stream = jk_avformat_stream(output_context.0, stream_index as u32);
                if in_stream.is_null() || out_stream.is_null() {
                    av_packet_unref(packet.0);
                    return Err(Self::infra_error("ストリーム情報の取得に失敗しました"));
                }

                av_packet_rescale_ts(
                    packet.0,
                    jk_avstream_time_base(in_stream),
                    jk_avstream_time_base(out_stream),
                );
                jk_av_packet_set_pos(packet.0, -1);

                let write_ret = av_interleaved_write_frame(output_context.0, packet.0);
                av_packet_unref(packet.0);
                if write_ret < 0 {
                    return Err(Self::infra_error(format!(
                        "av_interleaved_write_frame failed: {}",
                        Self::ff_error(write_ret)
                    )));
                }

                if input_size > 0 && !in_pb.is_null() {
                    let position = jk_avio_tell(in_pb);
                    let ratio = (position as f64 / input_size as f64).clamp(0.0, 1.0) as f32;
                    let bucket = (ratio * 100.0).floor() as i32;
                    if bucket > last_reported_bucket {
                        last_reported_bucket = bucket;
                        on_progress(DecryptionProgress { filename: filename.clone(), ratio });
                    }
                }
            }

            let trailer_ret = av_write_trailer(output_context.0);
            if trailer_ret < 0 {
                return Err(Self::infra_error(format!(
                    "av_write_trailer failed: {}",
                    Self::ff_error(trailer_ret)
                )));
            }

            on_progress(DecryptionProgress { filename, ratio: 1.0 });
        }

        Ok(output)
    }
}
