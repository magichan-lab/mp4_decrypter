#include <libavcodec/codec_par.h>
#include <libavcodec/packet.h>
#include <libavformat/avformat.h>
#include <libavutil/error.h>
#include <libavutil/rational.h>

int jk_averror_eof(void) {
    return AVERROR_EOF;
}

unsigned int jk_avformat_nb_streams(const AVFormatContext *ctx) {
    return ctx->nb_streams;
}

AVStream *jk_avformat_stream(const AVFormatContext *ctx, unsigned int index) {
    return ctx->streams[index];
}

AVIOContext *jk_avformat_pb(const AVFormatContext *ctx) {
    return ctx->pb;
}

void jk_avformat_set_pb(AVFormatContext *ctx, AVIOContext *pb) {
    ctx->pb = pb;
}

int jk_avformat_oformat_flags(const AVFormatContext *ctx) {
    return ctx->oformat ? ctx->oformat->flags : 0;
}

AVCodecParameters *jk_avstream_codecpar(const AVStream *st) {
    return st->codecpar;
}

AVRational jk_avstream_time_base(const AVStream *st) {
    return st->time_base;
}

void jk_avstream_set_time_base(AVStream *st, AVRational tb) {
    st->time_base = tb;
}

void jk_avcodecpar_set_codec_tag_zero(AVCodecParameters *par) {
    if (par) {
        par->codec_tag = 0;
    }
}

AVPacket *jk_av_packet_alloc(void) {
    return av_packet_alloc();
}

void jk_av_packet_free(AVPacket **pkt) {
    av_packet_free(pkt);
}

int jk_av_packet_stream_index(const AVPacket *pkt) {
    return pkt->stream_index;
}

void jk_av_packet_set_pos(AVPacket *pkt, int64_t pos) {
    pkt->pos = pos;
}

int64_t jk_avio_tell(AVIOContext *pb) {
    return avio_tell(pb);
}
