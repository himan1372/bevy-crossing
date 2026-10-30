# Arrival turn toward Porter and carriage floor clearance

Philip reports that the hop now lands in the correct spot, but the player snaps
to face Porter and a foot dips slightly through the train floor during hopping.
Scope: ease the player turn and preserve floor clearance during the departure
pose without shifting the accepted landing. Use isolated Temp runtime evidence;
protect the real save, no tests without request, sequential Cargo gates.

Investigation: Stage::Turn chases Porter's yaw but directly assigns player
Facing. GetOff also interpolates the carriage-to-platform base height from the
first clip frame, lowering the planted pose before its hop clears the carriage.

## Change

`m_player_main_talk.c_inc::Player_actor_Movement_Talk` eases the player's turn
with CALC_EASE(0.5), min 50 and max 2500 binary-angle units. The arrival now
advances that turn alongside Porter's existing chase and waits for both to face
one another before opening the greeting. It no longer assigns player facing
directly when Porter finishes turning.

The get-off base height stays at carriage height for the first quarter of the
clip, then uses smoothstep to settle during its middle half. The original root
motion and joint pose remain; the accepted final position is unchanged. This
base-height timing is a presentation adjustment for the port's carriage/platform
height difference, not a claim that retail uses this interpolation.

check/clippy/fmt and capture/default/release builds passed. No tests added or run.

## Verification

Isolated `%TEMP%/bevy-arrival-turn-floor-20261030`: checked full-size early
hop frames getoff-077/078.png and turn-floor-contact.png. Foot is visible above
the carriage floor during the crouch; model yaw samples after landing progress
0 -> -19.17 -> -69.46 -> -75.81 -> -76.71 degrees rather than one assignment.
The root stays at (-1.6531,5,-43.3) throughout turning. The unchanged walk ends
at (-0.5,5,-42.6), KeyA moves to (-1.9258,5,-42.0002), and F5 writes only the
synthetic save. Early planted base height is held while the root's native crouch
and hop arc remain. No new ERROR; existing iam_sugi_kpool startup warning only.

check/clippy/fmt/capture/default/release builds passed (release 3m53s).
The owned preview is closed. One focused review found no blocking issues.
Real save SHA256 remains C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.
Philip's acceptance remains open; this does not start the Nook/house-choice slice.
