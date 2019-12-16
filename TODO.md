current setup:
Middle drag -> panning
S-Left drag -> panning
Left drag -> select
Scroll -> zoom in/out
Escape -> quit
Up -> zoom in
Down -> zoom out
Left -> stride - 8b
Right -> stride + 8b
Grave -> endian swap
S -> string annotations

new setup:
Middle drag -> panning *DONE*
S-Left drag -> panning *DONE*
Left click -> select byte
S-Left click -> select word
+/- -> word size
LRUD -> move selection (start)
S-LRUS -> move selection (end)
Grave -> toggle endianness (BE, 16b LE, 32b LE, 64b LE, custom word)
W -> "where" mode (selection start by hex)
S-W -> "where" end mode (selection end by hex)

* Add height selection/
* tweak selection w/ kb
* type in hex address for start, end, len
* export selected region
* magic number annotator?
* search for hex sequence
* 16, 32, 64 bit endian cycling
* pop out selected data or annotation
* kb combo for panning
* refactor mouse interaction?
* refactor zoom?

Status panel:

+--------------+----------+
|  data        |  status  |
| (resizeable) | (fixed)  |
|              |          |
+--------------+----------+


UI interaction:
* left drag: selection (?)
* shift-left drag or middle drag: move window
* left click: clear selection and select annotation
* scroll wheel: zoom in and out
