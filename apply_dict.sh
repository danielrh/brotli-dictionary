#!/bin/sh
export XDICT=$1
export XDIR=$2

cat <<EOF > $XDIR/common/dictionary.c

#include "./dictionary.h"

#if defined(__cplusplus) || defined(c_plusplus)
extern "C" {
#endif

static const BrotliDictionary kBrotliDictionary = {
  /* size_bits_by_length */
  {
    0, 0, 0, 0, 10, 10, 11, 11,
    10, 10, 10, 10, 10, 9, 9, 8,
    7, 7, 8, 7, 7, 6, 6, 5,
    5, 0, 0, 0, 0, 0, 0, 0
  },

  /* offsets_by_length */
  {
    0, 0, 0, 0, 0, 4096, 9216, 21504,
    35840, 44032, 53248, 63488, 74752, 87040, 93696, 100864,
    104704, 106752, 108928, 113536, 115968, 118528, 119872, 121280,
    122016, 122784, 122784, 122784, 122784, 122784, 122784, 122784
  },

  /* data */
{
EOF
cat $XDICT | xxd -i >> $XDIR/common/dictionary.c
cat <<EOF >> $XDIR/common/dictionary.c
}
};

const BrotliDictionary* BrotliGetDictionary() {
  return &kBrotliDictionary;
}

#if defined(__cplusplus) || defined(c_plusplus)
}  /* extern "C" */
#endif 
EOF

cat <<EOF > $XDIR/enc/static_dict_lut.h
 /* Copyright 2015 Google Inc. All Rights Reserved.

   Distributed under MIT license.
   See file LICENSE for detail or copy at https://opensource.org/licenses/MIT
*/

/* Lookup table for static dictionary and transforms. */

#ifndef BROTLI_ENC_STATIC_DICT_LUT_H_
#define BROTLI_ENC_STATIC_DICT_LUT_H_

#include <brotli/types.h>

#if defined(__cplusplus) || defined(c_plusplus)
extern "C" {
#endif

typedef struct DictWord {
  /* Highest bit is used to indicate end of bucket. */
  uint8_t len;
  uint8_t transform;
  uint16_t idx;
} DictWord;

EOF

cat $XDICT | target/debug/brotli-dictionary -c >>  $XDIR/enc/static_dict_lut.h
cat <<EOF >> $XDIR/enc/static_dict_lut.h
#if defined(__cplusplus) || defined(c_plusplus)
}  /* extern "C" */
#endif

#endif  /* BROTLI_ENC_STATIC_DICT_LUT_H_ */

EOF


cat <<EOF >$XDIR/enc/dictionary_hash.c

/* Hash table on the 4-byte prefixes of static dictionary words. */

#include <brotli/port.h>
#include "./dictionary_hash.h"

#if defined(__cplusplus) || defined(c_plusplus)
extern "C" {
#endif
EOF
cat $XDICT | target/debug/brotli-dictionary -c -h >>  $XDIR/enc/static_dict_lut.h

cat <<EOF >>$XDIR/enc/dictionary_hash.c

#if defined(__cplusplus) || defined(c_plusplus)
}  /* extern "C" */
#endif

EOF
