# Bundled FFmpeg location

Place the Windows x64 FFmpeg executable at:

`Native/win-x64/ffmpeg.exe`

The build script copies this directory into the Everest mod package. If the
file is absent, the recorder falls back to `ffmpeg` on `PATH`, or the explicit
`FfmpegPath` setting.

For audio, configure `RecordingAudioDevice` to a DirectShow capture device.
For `SfxOnlyWithPostMix`, route Celeste SFX (without the music bus) to that
device and configure `BgmEventMapFile`; see the main README.

