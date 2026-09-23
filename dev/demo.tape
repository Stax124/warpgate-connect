# Rendered by `dev/scaffold.sh demo` against the seeded local Warpgate.
Output data/demo/warpgate-connect.gif

Set Shell bash
Set FontSize 18
Set Width 1240
Set Height 680
Set Padding 16
Set TypingSpeed 80ms

Hide
Type "export PATH=/vhs/target/x86_64-unknown-linux-musl/release:$PATH"
Enter
Type "alias warpgate-connect='warpgate-connect --skip-update --config data/config.toml' && clear"
Enter
Show

Type "warpgate-connect"
Sleep 500ms
Enter
Sleep 2s
Screenshot data/demo/targets.png
Sleep 500ms

Down 3
Sleep 1s

Type "prod"
Sleep 1.5s
Screenshot data/demo/search.png
Sleep 500ms
Escape
Sleep 1s

Ctrl+G
Sleep 1s
Type "data"
Sleep 1s
Screenshot data/demo/group-picker.png
Sleep 500ms
Enter
Sleep 1.5s

Down
Enter
Sleep 1.5s
Screenshot data/demo/connect-modal.png
Sleep 500ms
Escape
Sleep 1s

Hide
Ctrl+C
Sleep 500ms
