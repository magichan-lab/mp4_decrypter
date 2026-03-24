# Third-Party Notices

## FFmpeg

This application uses FFmpeg as an external library via dynamic or static linking.

- Library: FFmpeg
- Upstream: https://ffmpeg.org/
- License: GNU Lesser General Public License, version 2.1 or later (LGPL-2.1-or-later)
- Linking model: Dynamic or Static linking

### License notice

FFmpeg is not covered by this repository's MIT license. FFmpeg remains licensed under
its own LGPL-2.1-or-later terms.

When distributing this application together with FFmpeg shared libraries, you must keep
FFmpeg's copyright and license notices and comply with LGPL requirements, including but
not limited to the user's ability to replace the FFmpeg shared libraries used by the
application.

### Practical compliance notes for this repository

- This repository's own source code is licensed under MIT.
- The application is designed to use FFmpeg shared libraries from either:
  - `$FFMPEG_DIR/include` and `$FFMPEG_DIR/lib`, or
  - `third_party/ffmpeg/include` and `third_party/ffmpeg/lib`.
- The build uses dynamic or static linking to FFmpeg (`avformat`, `avcodec`, `avutil`,
  `swresample`, `swscale`), so end users can replace the FFmpeg shared libraries with
  compatible builds.
- If you redistribute FFmpeg binaries together with this application, you should also
  provide the corresponding FFmpeg source code or a clear written offer / link to obtain it,
  and include the FFmpeg LGPL license text in your redistribution package.

### Recommended redistribution artifacts

If you publish binaries, include at least the following alongside them:

- `LICENSE` (MIT for this repository)
- `THIRD_PARTY_NOTICES.md` (this file)
- FFmpeg's LGPL license text
- a link to the FFmpeg source code used for the shipped binaries, or the source archive itself
