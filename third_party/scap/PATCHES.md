# Local compatibility patch

This directory vendors `CapSoftware/scap` at commit
`c03f15a4631f40cd8d2e36807befb83b314cfeb7` (MIT).

The vendored source resolves `windows-capture` 1.5, where the capture-frame
timestamp accessor was renamed. The local patch uses `Frame::timestamp()` in
place of the removed `Frame::timespan()` method.

The Windows BGRA path also requests a no-padding CPU buffer. `scap` does not
expose row pitch on `BGRAFrame`, so returning the raw padded D3D staging buffer
would make consumers interpret padding bytes as the next scanline. This is an
intentional CPU copy; the recorder's memory bound comes from its fixed frame
queue rather than a zero-copy requirement.
