# Run from the workspace root with:
# vhs crates/ggsci-ratatui/examples/vhs/palette-gallery.tape

Output "target/palette-gallery.gif"

Require cargo

Set Shell "bash"
Set Theme "Raycast_Dark"
Set FontFamily "Berkeley Mono Variable"
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

Sleep 2s
Down@250ms 3
Sleep 2s

Right
Sleep 2s
Down@250ms 8
Sleep 2s

Right
Sleep 2s
Down@200ms 14
Sleep 2s

Right
Sleep 2s
Down@250ms 1
Sleep 2s

Right
Type "g"
Sleep 2s

Hide
Type "q"
