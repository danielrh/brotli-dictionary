use std::env;
use std::io;
extern crate brotli_decompressor;
use brotli_decompressor::dictionary::kBrotliDictionarySizeBitsByLength;
use brotli_decompressor::dictionary::kBrotliDictionaryOffsetsByLength;

pub static kInvalidMatch: u32 = 0xfffffff;

pub const kDictNumBits: i32 = 15;

pub static kDictHashMul32: u32 = 0x1e35a7bd;
use std::io::Read;

#[derive(Copy,Clone,Debug)]
pub struct DictWord {
    pub len: u8,
    pub transform: u8,
    pub idx: u16,
}
#[inline(always)]
pub fn BROTLI_UNALIGNED_LOAD32(sl: &[u8]) -> u32 {
  let mut p = [0u8;4];
  p[..].clone_from_slice(&sl.split_at(4).0);
  return (p[0] as u32) | ((p[1] as u32) << 8) | ((p[2] as u32) << 16) | ((p[3] as u32) << 24);
}
fn populate_dict_word(dictionary:&[u8]) -> (Vec<DictWord>, [u32; 1<< kDictNumBits], [i32; 1<< kDictNumBits],) {
    let mut dictionary_words = Vec::<DictWord>::new();
    let mut dictionary_buckets = [0u32; 1<<kDictNumBits];
    let mut dictionary_list = Vec::<Vec<DictWord>>::new();
    let mut static_dictionary_hash = [0i32; 1 << kDictNumBits];
    for i in 0..(1<< kDictNumBits) {
        dictionary_list.push(Vec::<DictWord>::new());
    }
    for word_size in 0..3 {
        assert_eq!(kBrotliDictionarySizeBitsByLength[word_size], 0);
    }
    for transform in [0u8, 10u8, 11u8].iter() {
        for word_size in 4..kBrotliDictionarySizeBitsByLength.len() {
            let word_size = word_size as u8 as usize;
            if kBrotliDictionarySizeBitsByLength[word_size] == 0 {
                continue;
            }
            let start = kBrotliDictionaryOffsetsByLength[word_size] as usize;
            let end = start + word_size * (1 << kBrotliDictionarySizeBitsByLength[word_size]);
            assert!(end <= dictionary.len() && start < dictionary.len());
            if word_size == kBrotliDictionarySizeBitsByLength.len() - 1 {
                assert_eq!(end, dictionary.len());
            }
            assert!(end >= word_size);
            let dict_sub_area = &dictionary[start..end];
            for i in 0..(1usize << kBrotliDictionarySizeBitsByLength[word_size]) {
                let word = &dict_sub_area[(i * word_size) .. ((i + 1) * word_size)];
                let mut transformed_word = [0u8;128];
                let final_size = brotli_decompressor::transform::TransformDictionaryWord(&mut transformed_word[..],
                                                         word,
                                                         word_size as i32,
                                                         *transform as i32) as usize;
                if final_size < 4 {
                    continue;
                }
                let h: u32 = BROTLI_UNALIGNED_LOAD32(&transformed_word[..]);
                let hash_result = ((h.wrapping_mul(kDictHashMul32) >> (32 - kDictNumBits))& ((1<<kDictNumBits) - 1)) as usize;
                if static_dictionary_hash[hash_result] == 0 {
                    static_dictionary_hash[hash_result] = ((i as i32) << 5) | word_size as i32
                }
                let pp = DictWord{len:word_size as u8,
                             transform:*transform,
                             idx:i as u16};
                //if dictionary_list[hash_result].len() > 10 {
                //   println!("{:?} {:?} -> {} maps to {} ({}) :: {:?}\n", word, &transformed_word[..final_size], h, hash_result, dictionary_list[hash_result].len(), pp);
                //}
                dictionary_list[hash_result].push(
                    pp);
            }
        }
    }
    dictionary_words.push(DictWord{len:0, transform:0, idx:0});
    for (i, dlist) in dictionary_list.iter().enumerate() {
        if dlist.len() == 0 {
            dictionary_buckets[i] = 0;
            continue;
        }
        dictionary_buckets[i] = dictionary_words.len() as u32;
        for bucket_element in dlist.iter() {
            dictionary_words.push(*bucket_element);
        }
        let last = dictionary_words.len() - 1;
        dictionary_words[last].len |= 128;
    }
    (dictionary_words,dictionary_buckets, static_dictionary_hash)
}

fn print_rust(dictionary_words: Vec<DictWord>,
              dictionary_buckets: [u32; 1<< kDictNumBits],
              static_dictionary_hash: [i32; 1<< kDictNumBits]) {
    println!("pub static kStaticDictionaryBuckets: [u16; {}] = ", 1<< kDictNumBits);
    println!("{:?}", &dictionary_buckets[..]);
    println!(";");
    println!("pub static kStaticDictionaryWords: [DictWord; {}] = ", dictionary_words.len());
    println!("{:?}", &dictionary_words[..]);
    println!(";");
    println!("pub static kStaticDictionaryHash: [u16; {}] = ", 1 << kDictNumBits);
    println!("{:?}", &static_dictionary_hash[..]);
    println!(";");
}

fn print_c(dictionary_words: Vec<DictWord>,
           dictionary_buckets: [u32; 1<< kDictNumBits],
           static_dictionary_hash: [i32; 1<< kDictNumBits]) {

    println!("static const int kDictNumBits = {};", kDictNumBits);
    println!("static const uint32_t kDictHashMul32 = 0x{:x};", kDictHashMul32);

    println!("static const uint32_t kStaticDictionaryBuckets [{}] = {}", 1<< kDictNumBits, "{");
    for index in 0..(1<<kDictNumBits) {
        if index + 1 == (1<<kDictNumBits) {
            println!("{}", dictionary_buckets[index]);
        } else {
            println!("{},", dictionary_buckets[index]);
        }
    }
    println!("{};", "}");
    println!("static const DictWord kStaticDictionaryWords[{}] = {}", dictionary_words.len(), "{");
    let dict_word_len = dictionary_words.len();
    for index in 0..dict_word_len {
        println!("{}{},{},{}{}","{",
                 dictionary_words[index].len,
                 dictionary_words[index].transform,
                 dictionary_words[index].idx,
                 if index + 1 == dict_word_len {"}"} else {"},"})
    }
    println!("{};", "}");
}
fn print_c_hash(dictionary_words: Vec<DictWord>,
           dictionary_buckets: [u32; 1<< kDictNumBits],
           static_dictionary_hash: [i32; 1<< kDictNumBits]) {
    println!("BROTLI_INTERNAL const uint16_t kStaticDictionaryHash [{}] = {}", 1 << kDictNumBits, "{");
    for index in 0..(1<<kDictNumBits) {
        if index + 1 == (1<<kDictNumBits) {
            println!("{}", static_dictionary_hash[index]);
        } else {
            println!("{},", static_dictionary_hash[index]);
        }
    }
    println!("{};", "}");
}
fn main() {
    let mut stdin = io::stdin();
    let mut dict = Vec::<u8>::new();
    let size = stdin.lock().read_to_end(&mut dict).unwrap();
    assert_eq!(size, dict.len());
    let (a, b, c) = populate_dict_word(&dict[..]);
    let mut do_c = false;
    let mut do_c_hash = false;
    for argument in env::args().skip(1) {
        if argument == "-c" {
            do_c = true;
        }
        if argument == "-h" {
            do_c_hash = true;
        }
    }
    if do_c {
        if do_c_hash {
            print_c_hash(a,b,c);
        } else {
            print_c(a,b,c);
        }
    } else {
        print_rust(a,b,c);
    }
}
