# Run from the workspace root with:
# vhs crates/ggsci-ratatui/examples/vhs/palette-gallery.tape

Output "target/palette-gallery.gif"

Require cargo

Set Shell "bash"
Set Theme "Raycast_Dark"
Set FontFamily "Liga Paper Mono"
Set FontSize 14
Set Width 1280
Set Height 720
Set Padding 16
Set Framerate 60
Set CursorBlink false

Env NO_COLOR ""

Hide
Type "cargo run --quiet -p ggsci-ratatui --example palette-gallery"
Enter
Wait+Screen /ggsci-rs palette gallery/
Show

Sleep 800ms
Down@125ms 6
Sleep 300ms

Right
Sleep 400ms
Down@100ms 6
Sleep 300ms

Right
Sleep 400ms
Down@75ms 10
Type "g"
Sleep 300ms

Right
Sleep 400ms
Down@125ms 1
Sleep 300ms

Right
Type "g"
Sleep 400ms

Hide
Type "q"
