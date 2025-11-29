//! Core Deflate (zlib) decompression implementation.
//! 
//! Includes BitReader, Huffman Tree construction/decoding (canonical),
//! and the main inflate loop handling stored, fixed, and dynamic blocks.

pub struct Header {
    data: [u8; 2],
}

pub struct Adler32 {
    data: u32,
}

/// Reads bits from a byte stream, LSB first.
pub struct BitReader {
    data: Vec<u8>,
    position: usize, // Current byte index in `data`
    bit_buffer: u64, // Accumulator for bits
    bits_left: u8,   // Number of valid bits in `bit_buffer`
}

impl BitReader {
    pub fn new(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
            position: 0,
            bit_buffer: 0,
            bits_left: 0,
        }
    }

    /// Ensures at least `n` bits are in the buffer.
    fn ensure_bits(&mut self, n: u8) {
        while self.bits_left < n {
            if self.position < self.data.len() {
                let byte = self.data[self.position] as u64;
                self.bit_buffer |= byte << self.bits_left;
                self.position += 1;
                self.bits_left += 8;
            } else {
                break; 
            }
        }
    }

    pub fn read_bits(&mut self, n: u8) -> u32 {
        self.ensure_bits(n);
        let result = (self.bit_buffer & ((1 << n) - 1)) as u32;
        self.bit_buffer >>= n;
        self.bits_left = self.bits_left.saturating_sub(n);
        result
    }

    pub fn peek_bits(&mut self, n: u8) -> u32 {
        self.ensure_bits(n);
        (self.bit_buffer & ((1 << n) - 1)) as u32
    }

    pub fn consume_bits(&mut self, n: u8) {
        self.bit_buffer >>= n;
        self.bits_left = self.bits_left.saturating_sub(n);
    }

    pub fn align_byte(&mut self) {
        let bits_to_skip = self.bits_left % 8;
        self.consume_bits(bits_to_skip);
    }
}

#[derive(Debug)]
pub struct HuffmanTree {
    pub counts: [u16; 16], // Number of codes of each length
    pub symbols: Vec<u16>, // Symbols sorted by code length
    
    // Optimizations for faster decoding (precomputed during from_lengths)
    pub min_code_value_for_length: [u16; 16], // The smallest canonical code value for a given length
    pub val_ptrs: [usize; 16], // Index into `symbols` for the first symbol of that length
}

impl HuffmanTree {
    pub fn new_fixed_literal() -> Self {
        let mut lengths = vec![0u8; 288];
        for i in 0..=143 { lengths[i] = 8; }
        for i in 144..=255 { lengths[i] = 9; }
        for i in 256..=279 { lengths[i] = 7; }
        for i in 280..=287 { lengths[i] = 8; }
        Self::from_lengths(&lengths).expect("Fixed literal tree valid")
    }

    pub fn new_fixed_distance() -> Self {
        let lengths = vec![5u8; 32];
        Self::from_lengths(&lengths).expect("Fixed distance tree valid")
    }

    pub fn from_lengths(lengths: &[u8]) -> Result<Self, String> {
        let mut counts = [0u16; 16];
        for &len in lengths {
            if len > 15 { return Err("Code length too long".to_string()); }
            if len > 0 {
                counts[len as usize] += 1;
            }
        }

        // Calculate min_code_value_for_length (also known as 'code' in RFC 1951, 3.2.2. step 2)
        let mut min_code_value_for_length = [0u16; 16];
        let mut current_code_val = 0;
        for bits in 1..=15 {
            current_code_val = (current_code_val + counts[bits-1]) << 1;
            min_code_value_for_length[bits] = current_code_val;
        }

        // Sort symbols by length and then by value (canonical Huffman: RFC 1951, 3.2.2. step 3)
        // We use a temporary vec of vecs to group symbols by length
        let mut symbols_grouped_by_len: Vec<Vec<u16>> = vec![Vec::new(); 16];
        for (sym_idx, &len) in lengths.iter().enumerate() {
            if len > 0 {
                // Symbols must be ordered by their original value *within* each length group
                // Since `lengths` (input) is indexed by symbol, and we fill `symbols_grouped_by_len` in order,
                // they are effectively already sorted.
                symbols_grouped_by_len[len as usize].push(sym_idx as u16);
            }
        }
        
        let mut symbols = Vec::new();
        let mut val_ptrs = [0usize; 16];
        let mut current_ptr = 0; // Tracks start index in `symbols` for current length
        
        for len in 1..=15 {
            val_ptrs[len] = current_ptr; // Store the start index for symbols of `len`
            if !symbols_grouped_by_len[len].is_empty() {
                symbols.extend_from_slice(&symbols_grouped_by_len[len]);
                current_ptr += symbols_grouped_by_len[len].len();
            }
        }

        Ok(Self { counts, symbols, min_code_value_for_length, val_ptrs })
    }

    pub fn decode(&self, reader: &mut BitReader) -> Result<u16, String> {
        let mut current_code_val_from_stream: u16 = 0; // The canonical code value read from the stream
        
        for len in 1..=15 { // Iterate through possible code lengths
            // Read one more bit and append to our current code pattern
            current_code_val_from_stream = (current_code_val_from_stream << 1) | reader.read_bits(1) as u16;

            // Check if there are codes of this length, and if our `current_code_val_from_stream`
            // falls within the range of valid canonical codes for this length.
            if self.counts[len] > 0 {
                let min_code = self.min_code_value_for_length[len];
                if current_code_val_from_stream >= min_code {
                    let offset = current_code_val_from_stream - min_code;
                    // Crucial fix: Ensure the code is within the count of codes for this length.
                    // Without this, a prefix of a longer code could be mistaken for a valid code of this length.
                    if offset < self.counts[len] {
                        let symbol_index = self.val_ptrs[len] + offset as usize;
                        if symbol_index < self.symbols.len() {
                            return Ok(self.symbols[symbol_index]);
                        }
                    }
                }
            }
        }
        Err("Symbol not found in tree or stream ended unexpectedly".to_string())
    }
}

/// Decompresses a raw DEFLATE stream (without zlib header/footer usually, or skipping it).
pub fn decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    // Skip Zlib header (2 bytes) if present
    let deflate_data = if data.len() > 2 { &data[2..] } else { data };
    let mut reader = BitReader::new(deflate_data);
    let mut output = Vec::new();
    
    loop {
        let bfinal = reader.read_bits(1) != 0;
        let btype = reader.read_bits(2);

        match btype {
            0 => process_stored_block(&mut reader, &mut output)?,
            1 => process_fixed_block(&mut reader, &mut output)?,
            2 => process_dynamic_block(&mut reader, &mut output)?,
            _ => return Err(format!("Invalid block type: {}", btype)),
        }

        if bfinal { break; }
    }
    Ok(output)
}

fn process_stored_block(reader: &mut BitReader, output: &mut Vec<u8>) -> Result<(), String> {
    reader.align_byte();
    let len = reader.read_bits(16) as u16;
    let nlen = reader.read_bits(16) as u16;
    if len != !nlen { return Err(format!("Stored block length mismatch")); }
    for _ in 0..len { output.push(reader.read_bits(8) as u8); }
    Ok(())
}

fn process_fixed_block(reader: &mut BitReader, output: &mut Vec<u8>) -> Result<(), String> {
    let lit_tree = HuffmanTree::new_fixed_literal();
    let dist_tree = HuffmanTree::new_fixed_distance();
    decode_block(reader, output, &lit_tree, &dist_tree)
}

fn process_dynamic_block(reader: &mut BitReader, output: &mut Vec<u8>) -> Result<(), String> {
    let hlit = (reader.read_bits(5) + 257) as usize;
    let hdist = (reader.read_bits(5) + 1) as usize;
    let hclen = (reader.read_bits(4) + 4) as usize;

    let code_len_order = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
    let mut code_lengths = vec![0u8; 19];
    
    for i in 0..hclen {
        code_lengths[code_len_order[i]] = reader.read_bits(3) as u8;
    }

    let code_len_tree = HuffmanTree::from_lengths(&code_lengths)?;
    let mut all_lengths = Vec::with_capacity(hlit + hdist);
    
    while all_lengths.len() < hlit + hdist {
        let symbol = code_len_tree.decode(reader)?;
        match symbol {
            0..=15 => all_lengths.push(symbol as u8),
            16 => {
                let prev = *all_lengths.last().ok_or("Repeat code 16 with no previous length")?;
                let repeat = reader.read_bits(2) + 3;
                for _ in 0..repeat { all_lengths.push(prev); }
            }
            17 => {
                let repeat = reader.read_bits(3) + 3;
                for _ in 0..repeat { all_lengths.push(0); }
            }
            18 => {
                let repeat = reader.read_bits(7) + 11;
                for _ in 0..repeat { all_lengths.push(0); }
            }
            _ => return Err(format!("Invalid code length symbol: {}", symbol)),
        }
    }

    let lit_lengths = &all_lengths[..hlit];
    let dist_lengths = &all_lengths[hlit..];
    let lit_tree = HuffmanTree::from_lengths(lit_lengths)?;
    let dist_tree = HuffmanTree::from_lengths(dist_lengths)?;

    decode_block(reader, output, &lit_tree, &dist_tree)
}

fn decode_block(reader: &mut BitReader, output: &mut Vec<u8>, lit_tree: &HuffmanTree, dist_tree: &HuffmanTree) -> Result<(), String> {
    loop {
        let symbol = lit_tree.decode(reader)?;
        match symbol {
            0..=255 => output.push(symbol as u8),
            256 => return Ok(()),
            257..=285 => {
                let (length, extra_bits) = length_base(symbol);
                let extra = if extra_bits > 0 { reader.read_bits(extra_bits) } else { 0 };
                let actual_length = length + extra as usize;

                let dist_symbol = dist_tree.decode(reader)?;
                let (dist, dist_extra) = distance_base(dist_symbol);
                let dist_bits = if dist_extra > 0 { reader.read_bits(dist_extra) } else { 0 };
                let actual_dist = dist + dist_bits as usize;

                if output.len() < actual_dist { return Err("LZ77 distance too far back".to_string()); }
                let start = output.len() - actual_dist;
                for i in 0..actual_length { output.push(output[start + i]); }
            }
            _ => return Err(format!("Invalid literal/length symbol: {}", symbol)),
        }
    }
}

fn length_base(symbol: u16) -> (usize, u8) {
    match symbol {
        257 => (3, 0), 258 => (4, 0), 259 => (5, 0), 260 => (6, 0), 261 => (7, 0),
        262 => (8, 0), 263 => (9, 0), 264 => (10, 0), 265 => (11, 1), 266 => (13, 1),
        267 => (15, 1), 268 => (17, 1), 269 => (19, 2), 270 => (23, 2), 271 => (27, 2),
        272 => (31, 2), 273 => (35, 3), 274 => (43, 3), 275 => (51, 3), 276 => (59, 3),
        277 => (67, 4), 278 => (83, 4), 279 => (99, 4), 280 => (115, 4), 281 => (131, 5),
        282 => (163, 5), 283 => (195, 5), 284 => (227, 5), 285 => (258, 0),
        _ => (0, 0),
    }
}

fn distance_base(symbol: u16) -> (usize, u8) {
    match symbol {
        0 => (1, 0), 1 => (2, 0), 2 => (3, 0), 3 => (4, 0),
        4 => (5, 1), 5 => (7, 1), 6 => (9, 2), 7 => (13, 2),
        8 => (17, 3), 9 => (25, 3), 10 => (33, 4), 11 => (49, 4),
        12 => (65, 5), 13 => (97, 5), 14 => (129, 6), 15 => (193, 6),
        16 => (257, 7), 17 => (385, 7), 18 => (513, 8), 19 => (769, 8),
        20 => (1025, 9), 21 => (1537, 9), 22 => (2049, 10), 23 => (3073, 10),
        24 => (4097, 11), 25 => (6145, 11), 26 => (8193, 12), 27 => (12289, 12),
        28 => (16385, 13), 29 => (24577, 13),
        _ => (0, 0),
    }
}
