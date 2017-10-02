
* Add stride shifting [DONE]
* tweak selection w/ kb
* column height resizing on window resize [DONE]
* Status panel (slideable? sizeable? hideable? fixed?)
* persistent settings
* magic number annotator?
* pop out selected data or annotation
* single click to clear selection [DONE] or select annotation [still todo]


* ANNOTATIONS: recognize code chunks?
  * Architectures: x86, z80, x86_64, 6502, m68k, avr, arm, thumb, java bytecode?
    * Start with 8-bit or 16-bit processors
    * z80, 6502, m68k, 8080 to start
  * Approach: need to account for instruction alignment

Basically, we attempt to disassemble a chunk and run it through a verifier. If two adjacent chunks verify and are aligned, we merge them into one chunk. A "verified" chunk may consist of different types! We could use a markov filter to distinguish between: architectures, compilers, languages, etc.