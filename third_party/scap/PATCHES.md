# Local compatibility patch

This directory vendors `CapSoftware/scap` at commit
`c03f15a4631f40cd8d2e36807befb83b314cfeb7` (MIT).

The vendored source resolves `windows-capture` 1.5, where the capture-frame
timestamp accessor was renamed. The local patch uses `Frame::timestamp()` in
place of the removed `Frame::timespan()` method. No capture behavior is
otherwise changed.

