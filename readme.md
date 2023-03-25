# mimizu

mimizu is a single-stroke handwriting recognizer for VR (WIP).  You can enter characters by writing strokes in the air.

## Usage

mimizu works on the OpenVR (SteamVR), which is the only API that supports overlay applications (in my understanding).

Currently the strokes are mostly compatible with [Palm Graffiti](https://upload.wikimedia.org/wikipedia/commons/6/68/Palm_Graffiti_gestures.png).

- Press the grips and triggers of both hands to activate/deactivate the program.
- Both trigger and grip buttons can be used to write strokes.
- Two-stroke "X" is not supported.  Use a mirrored stroke of "K".
- Extended character mode is not supported.
- Write backslash (top-left to right-bottom line) to enter the numeric input mode.
- Write backslash reversely (right-bottom to top-left line) to enter the alphabetic input mode.

## Protocol

- [VRChat OSC Chatbox](https://docs.vrchat.com/docs/osc-as-input-controller) (UDP port 9000).
- (not implemented yet) Keyboard input emulation?
- (not implemented yet) WebSocket server?
