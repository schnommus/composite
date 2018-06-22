extern crate byteorder;

use std::fs::File;
use std::env;
use std::process;

use byteorder::*;

const BUF_SZ:               usize = 128;
const HALT_AT:              usize = 4096*1600;

const SAMPLE_RATE:            f32 = 20e6;

const H_LINE_TIME_SEC:        f32 = 64e-6;
const H_FRONT_PORCH_SEC:      f32 = 1.65e-6;
const H_SYNC_PULSE_SEC:       f32 = 4.7e-6;
const H_BACK_PORCH_SEC:       f32 = 5.7e-6;
const H_ACTIVE_VIDEO_SEC:     f32 = 51.96e-6;

const V_SYNC_SECTION_SEC:     f32 = H_LINE_TIME_SEC/2.0;
const V_SHORT_SYNC_PULSE_SEC: f32 = 2.35e-6;
const V_BROAD_SYNC_PULSE_SEC: f32 = 4.7e-6;

const SYNC_THRESHOLD:         f32 = 0.07;
const SYNC_LEN_DELTA:         f32 = 0.5e-6;


struct CompositeDecode {
    input_buffer: [f32; BUF_SZ],
    input_index: usize,

    in_sync_pulse: bool,
    since_edge: usize,
}

impl CompositeDecode {
    fn new() -> CompositeDecode {
        let input_buffer = [0.0; BUF_SZ];
        let input_index = 0;
        let in_sync_pulse = false;
        let since_edge = 0;
        CompositeDecode {
            input_buffer,
            input_index,
            in_sync_pulse,
            since_edge,
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

        let buf = self.input_buffer;

        let mut pos = 0;

        // This loop is basically an edge detector with hysteresis
        loop {
            let n_consumed = buf[pos..].iter().take_while(
                |v| match self.in_sync_pulse {
                    true => **v < SYNC_THRESHOLD * 1.5,
                    false => **v > SYNC_THRESHOLD * 0.5
                }).count();

            pos += n_consumed;
            self.since_edge += n_consumed;

            if pos < BUF_SZ {
                // Not at the end of the buffer? means we hit an edge
                self.in_sync_pulse = !self.in_sync_pulse;

                // Just came out of a sync pulse - record its length
                if self.in_sync_pulse == false {
                    let pulse_len_sec = self.since_edge as f32 * (1./SAMPLE_RATE);

                    print!("sync pulse: {} usec [{} samples] - ",
                           pulse_len_sec * 1e6,
                           self.since_edge);


                    let broad_sync_len = V_SYNC_SECTION_SEC - V_BROAD_SYNC_PULSE_SEC;
                    if (pulse_len_sec - H_SYNC_PULSE_SEC).abs() < SYNC_LEN_DELTA {
                        println!("HSYNC");
                    } else if (pulse_len_sec - V_SHORT_SYNC_PULSE_SEC).abs()
                               < SYNC_LEN_DELTA {
                        println!("VSYNC SHORT");
                    } else if (pulse_len_sec - broad_sync_len).abs() < SYNC_LEN_DELTA {
                        println!("VSYNC BROAD");
                    } else {
                        println!("SYNC [UNKNOWN]");
                    }

                }
                self.since_edge = 0;
            } else {
                break
            }
        }
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
