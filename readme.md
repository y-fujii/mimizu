# mimizu

mimizu is a single-stroke handwriting recognizer for VR.  You can enter
characters by writing strokes in the air.

## Usage

mimizu works on OpenVR (SteamVR), which is the only API that supports overlay
applications (as far as I know).

Currently the strokes are mostly compatible with
 [Palm Graffiti](https://upload.wikimedia.org/wikipedia/commons/6/68/Palm_Graffiti_gestures.png).

- Press the grips and triggers of both hands simultaneously to
  activate/deactivate the program.
- Either trigger or grip buttons can be used to write strokes.
- Two-stroke "X" is not supported.  Use a mirrored stroke of "K".
- Extended character mode is not supported.
- Write backslash (top-left to right-bottom line) to enter the numeric input
  mode.
- Write backslash reversely (right-bottom to top-left line) to enter the
  alphabetic input mode.

## Output protocols

- [VRChat OSC Chatbox](https://docs.vrchat.com/docs/osc-as-input-controller) (UDP port 9000).
- Keyboard input emulation (currently on Windows only).
- (not implemented yet) WebSocket server?
