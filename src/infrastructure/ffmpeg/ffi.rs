//! FFmpeg C API 最小バインディング定義
//!
//! 復号 remux に必要な型・定数・関数群

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::ffi::{c_char, c_int, c_uint};

pub type int64_t = i64;

pub const AVIO_FLAG_WRITE: c_int = 2;
pub const AVFMT_NOFILE: c_int = 0x0001;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct AVRational {
    pub num: c_int,
    pub den: c_int,
}

#[repr(C)]
pub struct AVDictionary {
    _private: [u8; 0],
}

#[repr(C)]
pub struct AVFormatContext {
    _private: [u8; 0],
}

#[repr(C)]
pub struct AVInputFormat {
    _private: [u8; 0],
}

#[repr(C)]
pub struct AVOutputFormat {
    _private: [u8; 0],
}

#[repr(C)]
pub struct AVIOContext {
    _private: [u8; 0],
}

#[repr(C)]
pub struct AVStream {
    _private: [u8; 0],
}

#[repr(C)]
pub struct AVCodec {
    _private: [u8; 0],
}

#[repr(C)]
pub struct AVCodecParameters {
    _private: [u8; 0],
}

#[repr(C)]
pub struct AVPacket {
    _private: [u8; 0],
}

unsafe extern "C" {
    pub fn av_strerror(errnum: c_int, errbuf: *mut c_char, errbuf_size: usize) -> c_int;

    pub fn av_dict_set(
        pm: *mut *mut AVDictionary,
        key: *const c_char,
        value: *const c_char,
        flags: c_int,
    ) -> c_int;

    pub fn av_dict_free(m: *mut *mut AVDictionary);

    pub fn avformat_open_input(
        ps: *mut *mut AVFormatContext,
        url: *const c_char,
        fmt: *mut AVInputFormat,
        options: *mut *mut AVDictionary,
    ) -> c_int;

    pub fn avformat_find_stream_info(
        ic: *mut AVFormatContext,
        options: *mut *mut AVDictionary,
    ) -> c_int;

    pub fn avformat_close_input(s: *mut *mut AVFormatContext);

    pub fn avformat_alloc_output_context2(
        ctx: *mut *mut AVFormatContext,
        oformat: *mut AVOutputFormat,
        format_name: *const c_char,
        filename: *const c_char,
    ) -> c_int;

    pub fn avformat_new_stream(s: *mut AVFormatContext, c: *const AVCodec) -> *mut AVStream;

    pub fn avformat_write_header(s: *mut AVFormatContext, options: *mut *mut AVDictionary)
        -> c_int;

    pub fn av_interleaved_write_frame(s: *mut AVFormatContext, pkt: *mut AVPacket) -> c_int;

    pub fn av_write_trailer(s: *mut AVFormatContext) -> c_int;

    pub fn avformat_free_context(s: *mut AVFormatContext);

    pub fn av_read_frame(s: *mut AVFormatContext, pkt: *mut AVPacket) -> c_int;

    pub fn av_packet_unref(pkt: *mut AVPacket);

    pub fn av_packet_rescale_ts(pkt: *mut AVPacket, tb_src: AVRational, tb_dst: AVRational);

    pub fn avcodec_parameters_copy(
        dst: *mut AVCodecParameters,
        src: *const AVCodecParameters,
    ) -> c_int;

    pub fn avio_open(s: *mut *mut AVIOContext, url: *const c_char, flags: c_int) -> c_int;
    pub fn avio_closep(s: *mut *mut AVIOContext) -> c_int;
    pub fn avio_size(s: *mut AVIOContext) -> int64_t;

    // C shim accessors
    pub fn jk_averror_eof() -> c_int;
    pub fn jk_avformat_nb_streams(ctx: *const AVFormatContext) -> c_uint;
    pub fn jk_avformat_stream(ctx: *const AVFormatContext, index: c_uint) -> *mut AVStream;
    pub fn jk_avformat_pb(ctx: *const AVFormatContext) -> *mut AVIOContext;
    pub fn jk_avformat_set_pb(ctx: *mut AVFormatContext, pb: *mut AVIOContext);
    pub fn jk_avformat_oformat_flags(ctx: *const AVFormatContext) -> c_int;

    pub fn jk_avstream_codecpar(st: *const AVStream) -> *mut AVCodecParameters;
    pub fn jk_avstream_time_base(st: *const AVStream) -> AVRational;
    pub fn jk_avstream_set_time_base(st: *mut AVStream, tb: AVRational);
    pub fn jk_avcodecpar_set_codec_tag_zero(par: *mut AVCodecParameters);

    pub fn jk_av_packet_alloc() -> *mut AVPacket;
    pub fn jk_av_packet_free(pkt: *mut *mut AVPacket);
    pub fn jk_av_packet_stream_index(pkt: *const AVPacket) -> c_int;
    pub fn jk_av_packet_set_pos(pkt: *mut AVPacket, pos: int64_t);

    pub fn jk_avio_tell(pb: *mut AVIOContext) -> int64_t;
}
