extern crate byteorder;

use std::fs::File;
use std::env;
use std::process;

use byteorder::*;

const BUF_SZ: usize = 128;
const HALT_AT: usize = 4096;

struct CompositeDecode {
    input_buffer: [f32; BUF_SZ],
    input_index: usize,
}

impl CompositeDecode {
    fn new() -> CompositeDecode {
        let input_buffer = [0.0; BUF_SZ];
        let input_index = 0;
        CompositeDecode {
            input_buffer,
            input_index
        }
    }

    fn push_data(&mut self, values: &[f32]) {

        /* Not worrying about huge input buffers for now... */
        assert!(values.len() < BUF_SZ);

        /* Copy as much as possible into the input buffer */

        let n_to_copy = std::cmp::min(values.len(),
                                      self.input_buffer.len() - self.input_index);
        self.input_buffer[self.input_index..self.input_index+n_to_copy]
            .copy_from_slice(&values[0..n_to_copy]);

        self.input_index += n_to_copy;

        /* Process if the buffer is full */

        if self.input_index == BUF_SZ {
            self.process();
            self.input_index = 0;

            /* Handle case where there wasn't enough space to copy all values */
            if n_to_copy < values.len() {
                let n_left = values.len() - n_to_copy;
                self.input_buffer[self.input_index..n_left]
                    .copy_from_slice(&values[values.len() - n_left..]);
                self.input_index += n_left;
            }
        }

    }

    fn process(&mut self) {
        println!("process");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: composite_decode file_name");
        process::exit(1);
    }

    let mut file = File::open(args[1].clone()).unwrap();

    let mut decoder = CompositeDecode::new();

    let mut halt_count = 0;

    while let Ok(v) = file.read_f32::<LittleEndian>() {
        decoder.push_data(&[v]);

        halt_count += 1;
        if halt_count > HALT_AT {
            break;
        }
    }
}
