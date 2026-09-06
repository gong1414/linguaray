#!/bin/sh

# PopClip supplies an already percent-encoded representation, so selected
# text never passes through a shell interpolation or a temporary file.
open -g "linguaray://translate?text=${POPCLIP_URLENCODED_TEXT}"
