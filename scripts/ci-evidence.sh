#!/usr/bin/env bash
# CI evidence: run the real binary; the exact result is the finding.
# Linux: call it as `xvfb-run -a bash scripts/ci-evidence.sh`; a window manager and one window are started here.
BIN=target/debug/fs-capture
mkdir -p evidence
if [ "$(uname -s)" = Linux ]; then
  openbox > evidence/wm.txt 2>&1 &
  sleep 2
  xterm -title fs-capture-test -e sleep 120 > /dev/null 2>&1 &
  sleep 2
fi
run() { name=$1; shift; "$BIN" "$@" > "evidence/$name.txt" 2>&1; echo "exit=$?" >> "evidence/$name.txt"; }
run screens screens
run windows windows
run shot shot --primary --dir evidence/out
[ "$(uname -s)" = Linux ] && run shot-screen shot --screen "$(head -1 evidence/screens.txt | cut -f1)" --dir evidence/out
[ "$(uname -s)" = Linux ] && run shot-window shot --title fs-capture-test --dir evidence/out
run zip zip evidence/out.zip --dir evidence/out
exit 0
