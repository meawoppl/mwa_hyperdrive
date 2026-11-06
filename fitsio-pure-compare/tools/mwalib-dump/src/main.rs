//! Read every visibility of an MWA observation through mwalib and print what
//! was read: the metafits/correlator metadata mwalib decoded, then a SHA-256
//! of each (timestep, coarse channel) buffer from `read_by_baseline` and
//! `read_by_frequency`. Two runs agree if and only if mwalib decoded the
//! same metadata and the same visibilities.
//!
//! Usage: mwalib-dump <metafits> <gpubox>...
use mwalib::{CorrelatorContext, GpuboxError};
use sha2::{Digest, Sha256};
use std::io::Write;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (metafits, gpuboxes) = args.split_first().expect("usage: mwalib-dump METAFITS GPUBOX...");
    let context = CorrelatorContext::new(metafits, gpuboxes)?;
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    writeln!(out, "{context}")?;
    for rf in &context.metafits_context.rf_inputs {
        writeln!(out, "{rf:?}")?;
    }

    let mut total = Sha256::new();
    let size = context.num_timestep_coarse_chan_floats;
    let mut buffer = vec![0f32; size];
    for t in 0..context.num_timesteps {
        for c in 0..context.num_coarse_chans {
            for (order, by_freq) in [("bl", false), ("freq", true)] {
                let result = if by_freq {
                    context.read_by_frequency_into_buffer(t, c, &mut buffer)
                } else {
                    context.read_by_baseline_into_buffer(t, c, &mut buffer)
                };
                match result {
                    Ok(()) => {
                        let bytes: Vec<u8> = buffer.iter().flat_map(|v| v.to_le_bytes()).collect();
                        let digest = Sha256::digest(&bytes);
                        total.update(digest);
                        writeln!(out, "t={t} c={c} {order} {}", hex(&digest))?;
                    }
                    Err(GpuboxError::NoDataForTimeStepCoarseChannel { .. }) => {
                        writeln!(out, "t={t} c={c} {order} no-data")?;
                    }
                    Err(e) => return Err(e.into()),
                }
            }
        }
    }
    writeln!(out, "total {}", hex(&total.finalize()))?;
    Ok(())
}
