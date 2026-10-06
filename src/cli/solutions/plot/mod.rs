// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Code to plot calibration solutions.

mod error;

pub(crate) use error::SolutionsPlotError;

use std::path::PathBuf;

use clap::Parser;

use crate::HyperdriveError;

#[derive(Parser, Debug, Default)]
pub(crate) struct SolutionsPlotArgs {
    #[clap(value_name = "SOLUTIONS_FILES")]
    files: Vec<PathBuf>,

    /// The reference tile to use. If this isn't specified, the best one
    /// from the end is used.
    #[clap(short, long)]
    ref_tile: Option<usize>,

    /// Don't use a reference tile. Using this will ignore any input for
    /// `ref_tile`.
    #[clap(short, long)]
    no_ref_tile: bool,

    /// Don't plot the leakage polarisations (D_x and D_y).
    #[clap(long)]
    ignore_cross_pols: bool,

    /// The minimum y-range value on the amplitude gain plots.
    #[clap(long)]
    min_amp: Option<f64>,

    /// The maximum y-range value on the amplitude gain plots.
    #[clap(long)]
    max_amp: Option<f64>,

    /// The number of rows to use in the plots. The default is determined based
    /// off of the number of tiles in the solutions.
    #[clap(long)]
    num_rows: Option<usize>,

    /// The number of columns to use in the plots. The default is determined
    /// based off of the number of tiles in the solutions.
    #[clap(long)]
    num_cols: Option<usize>,

    /// The directory to write the plots into. If this doesn't exist, then the
    /// relevant directories will be created. The filenames are based off of the
    /// input files, just as they would without specifying the output directory.
    #[clap(short, long)]
    output_directory: Option<String>,

    /// The metafits file associated with the solutions. This provides
    /// additional information on the plots, like the tile names.
    #[clap(short, long)]
    metafits: Option<PathBuf>,
}

impl SolutionsPlotArgs {
    pub(crate) fn run(self) -> Result<(), HyperdriveError> {
        plotting::plot_all_sol_files(self)?;
        Ok(())
    }
}

mod plotting {
    use std::str::FromStr;

    use log::{debug, info, warn};
    use marlu::Jones;
    use ndarray::prelude::*;
    use rizzma::{
        artist::{Patch, Rgba},
        core::{Affine2D, Path, PathCode},
        mathtext::layout_rich_text,
        text::FontSource,
        Figure,
    };
    use vec1::Vec1;

    use super::*;
    use crate::solutions::{ao, hyperdrive, CalSolutionType, CalibrationSolutions};

    // The plots are 3200x1800 pixels. Everything is drawn in pixel coordinates
    // (origin at the top left, as in an image) so that the output doesn't
    // depend on rizzma's built-in decorations, which can't be tuned to match
    // the layout that these plots have always had.
    const WIDTH: i32 = 3200;
    const HEIGHT: i32 = 1800;
    const DPI: f64 = 100.0;
    /// The height of the strip at the top of the plots that holds the title.
    const TITLE_STRIP: i32 = 58;
    /// The width of the dead margin to the left of the amplitude plots.
    const AMPS_MARGIN: i32 = 15;
    /// The height of the strip between a tile's name and its axis that holds
    /// the x tick labels.
    const X_LABEL_AREA: i32 = 15;
    const TICK_LENGTH: i32 = 5;
    const TICK_LABEL_EM: f64 = 9.6;
    /// Pixels between the end of a tick and its label.
    const X_LABEL_GAP: i32 = 5;
    const Y_LABEL_GAP: i32 = 3;

    /// Polarisation names, and their colours.
    const POLS: [(&str, &str, Rgba); 4] = [
        ("g", "X", Rgba::BLUE),
        (
            "D",
            "X",
            Rgba {
                a: 0.2,
                ..Rgba::BLUE
            },
        ),
        (
            "D",
            "Y",
            Rgba {
                a: 0.2,
                ..Rgba::RED
            },
        ),
        ("g", "Y", Rgba::RED),
    ];
    const FLAGGED: Rgba = Rgba {
        r: 220.0 / 255.0,
        g: 220.0 / 255.0,
        b: 220.0 / 255.0,
        a: 1.0,
    };
    /// Each grid line is translucent, so the crossings of two are darker.
    const GRID: Rgba = Rgba {
        a: 50.0 / 255.0,
        ..Rgba::BLACK
    };

    pub(crate) fn plot_all_sol_files(args: SolutionsPlotArgs) -> Result<(), SolutionsPlotError> {
        let SolutionsPlotArgs {
            files,
            ref_tile,
            no_ref_tile,
            ignore_cross_pols,
            min_amp,
            max_amp,
            num_rows,
            num_cols,
            output_directory,
            metafits,
        } = args;

        if files.is_empty() {
            return Err(SolutionsPlotError::NoInputs);
        }

        let mwalib_context = match metafits.as_deref() {
            Some(m) => Some(mwalib::MetafitsContext::new(m, None)?),
            None => None,
        };
        let mwalib_tile_names = match mwalib_context.as_ref() {
            Some(c) => {
                // TODO: Make mwalib provide SoA, not AoS
                let names = c
                    .antennas
                    .iter()
                    .map(|a| a.tile_name.clone())
                    .collect::<Vec<String>>();
                Some(
                    Vec1::try_from_vec(names)
                        .map_err(|_| SolutionsPlotError::MetafitsNoAntennaNames)?,
                )
            }
            None => None,
        };

        // Have we warned the user that tile names won't be on the plots?
        let mut warned_no_tile_names = false;

        for solutions_file in &files {
            debug!("Plotting solutions for '{}'", solutions_file.display());
            let solutions_file = solutions_file.canonicalize()?;
            let solutions_type = match solutions_file
                .extension()
                .and_then(|os_str| os_str.to_str())
                .and_then(|s| CalSolutionType::from_str(s).ok())
            {
                Some(sol_type) => sol_type,
                None => return Err(SolutionsPlotError::InvalidSolsFormat(solutions_file)),
            };
            let base = solutions_file
                .file_stem()
                .unwrap_or_else(|| {
                    panic!(
                        "Calibration solutions filename '{}' has no file stem",
                        solutions_file.display()
                    );
                })
                .to_str()
                .unwrap_or_else(|| {
                    panic!(
                        "Calibration solutions filename '{}' contains invalid UTF-8",
                        solutions_file.display()
                    )
                });
            let base = if let Some(o) = output_directory.as_deref() {
                let pb = PathBuf::from(o);
                if !pb.exists() {
                    std::fs::create_dir_all(&pb)?;
                }
                pb.join(base)
                    .to_str()
                    .expect("only contains valid UTF-8, as this has been checked above")
                    .to_string()
            } else {
                base.to_string()
            };

            let sols = match solutions_type {
                CalSolutionType::Fits => hyperdrive::read(&solutions_file)?,
                CalSolutionType::Bin => ao::read(&solutions_file)?,
            };
            let plot_title = format!(
                "obsid {}",
                sols.obsid
                    .or_else(|| mwalib_context.as_ref().map(|m| m.obs_id))
                    .map(|o| o.to_string())
                    .unwrap_or_else(|| "<unknown>".to_string())
            );
            let tile_names = sols.tile_names.as_ref().or(mwalib_tile_names.as_ref());
            if tile_names.is_none() && !warned_no_tile_names {
                // N.B. Not using `crate::cli::Warn` here because multiple
                // calibration solutions may be plotted, and we want the user to
                // see the warnings for each file.
                warn!("No metafits supplied; the obsid and tile names won't be on the plots");
                warned_no_tile_names = true;
            }

            // How should the plot be split up to distribute the tiles?
            let (auto_num_rows, auto_num_cols, tile_name_font_size) = {
                let total_num_tiles = sols.di_jones.len_of(Axis(1));
                let (num_rows, tile_name_font_size) = match total_num_tiles {
                    0..=128 => (8, 30),
                    129..=256 => (10, 24),
                    _ => (16, 18),
                };
                let num_cols = (total_num_tiles as f64 / num_rows as f64).ceil() as usize;
                (num_rows, num_cols, tile_name_font_size)
            };
            let plot_files = plotting::plot_sols(
                &sols,
                &base,
                &plot_title,
                ref_tile,
                no_ref_tile,
                tile_names,
                ignore_cross_pols,
                min_amp,
                max_amp,
                num_rows.unwrap_or(auto_num_rows),
                num_cols.unwrap_or(auto_num_cols),
                tile_name_font_size,
            )?;
            info!("Wrote {:?}", plot_files);
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn plot_sols(
        sols: &CalibrationSolutions,
        filename_base: &str,
        obs_name: &str,
        ref_tile: Option<usize>,
        no_ref_tile: bool,
        tile_names: Option<&Vec1<String>>,
        ignore_cross_pols: bool,
        min_amp: Option<f64>,
        max_amp: Option<f64>,
        num_rows: usize,
        num_cols: usize,
        tile_name_font_size: i32,
    ) -> Result<Vec<String>, rizzma::skia::PngError> {
        let (num_timeblocks, total_num_tiles, _) = sols.di_jones.dim();

        let mut amps = Array2::from_elem(
            (sols.di_jones.dim().1, sols.di_jones.dim().2),
            [0.0, 0.0, 0.0, 0.0],
        );
        let mut phases = Array2::from_elem(
            (sols.di_jones.dim().1, sols.di_jones.dim().2),
            [0.0, 0.0, 0.0, 0.0],
        );

        let ref_tile = match (no_ref_tile, ref_tile) {
            (true, _) => {
                debug!("Not using a reference tile");
                None
            }
            (_, Some(r)) => {
                debug!("Using user-specified reference tile: {r}");
                Some(r)
            }
            // If the reference tile wasn't defined, use the first valid one from
            // the end.
            (_, None) => {
                let possibly_good = sols
                    .di_jones
                    .slice(s![0_usize, .., ..])
                    // Search only in the first timeblock
                    .outer_iter()
                    // Search by tile from the end
                    .rev()
                    .enumerate()
                    // Include solutions for tiles that (1) aren't all NaN and
                    // (2) aren't singular (this can happen when dealing with
                    // single-pol data).
                    .filter(|(_, j)| !j.iter().all(|f| f.any_nan() || f.inv().any_nan()))
                    .map(|(i, _)| i)
                    .next();
                // If the search for a valid tile didn't find anything, all
                // solutions must be NaN. In this case, it doesn't matter what the
                // reference is.
                let r = possibly_good.map(|g| total_num_tiles - 1 - g);
                debug!("Automatically determined reference tile: {r:?}");
                r
            }
        };

        let font = FontSource::dejavu_sans();
        let mut output_filenames = vec![];
        for timeblock in 0..num_timeblocks {
            let (output_amps, output_phases) = if num_timeblocks > 1 {
                (
                    format!("{filename_base}_amps_{timeblock:03}.png"),
                    format!("{filename_base}_phases_{timeblock:03}.png"),
                )
            } else {
                (
                    format!("{filename_base}_amps.png"),
                    format!("{filename_base}_phases.png"),
                )
            };

            // Draw the reference tile number and the GPS times for this
            // timeblock.
            let mut meta_str = match ref_tile {
                Some(ref_tile) => format!("Ref. tile {ref_tile}"),
                None => String::new(),
            };
            let time_str = match (
                sols.start_timestamps
                    .as_ref()
                    .and_then(|t| t.get(timeblock)),
                sols.end_timestamps.as_ref().and_then(|t| t.get(timeblock)),
                sols.average_timestamps
                    .as_ref()
                    .and_then(|t| t.get(timeblock)),
            ) {
                (Some(s), Some(e), Some(a)) => {
                    format!(
                        "GPS start {}, end {}, average {}",
                        s.to_gpst_seconds(),
                        e.to_gpst_seconds(),
                        a.to_gpst_seconds()
                    )
                }
                (Some(s), Some(e), None) => format!(
                    "GPS start {}, end {}",
                    s.to_gpst_seconds(),
                    e.to_gpst_seconds()
                ),
                (Some(s), None, None) => format!("GPS start {}, end unknown", s.to_gpst_seconds()),
                (None, Some(e), None) => format!("GPS start unknown, end {}", e.to_gpst_seconds()),
                (Some(s), None, Some(a)) => format!(
                    "GPS start {}, end unknown, average {}",
                    s.to_gpst_seconds(),
                    a.to_gpst_seconds()
                ),
                (None, Some(e), Some(a)) => format!(
                    "GPS start unknown, end {}, average {}",
                    e.to_gpst_seconds(),
                    a.to_gpst_seconds()
                ),
                (None, None, Some(a)) => format!(
                    "GPS start unknown, end unknown, average {}",
                    a.to_gpst_seconds()
                ),
                (None, None, None) => String::new(),
            };
            if !meta_str.is_empty() && !time_str.is_empty() {
                meta_str.push_str(", ");
            }
            meta_str.push_str(&time_str);

            let ones = Array1::from_elem(sols.di_jones.dim().2, Jones::identity());
            let ref_jones = if let Some(ref_tile) = ref_tile {
                sols.di_jones.slice(s![timeblock, ref_tile, ..])
            } else {
                ones.view()
            };
            amps.outer_iter_mut()
                .zip(phases.outer_iter_mut())
                .zip(sols.di_jones.slice(s![timeblock, .., ..]).outer_iter())
                .for_each(|((mut a, mut p), s)| {
                    a.iter_mut()
                        .zip(p.iter_mut())
                        .zip(s.iter())
                        .zip(ref_jones.iter())
                        .for_each(|(((a, p), s), r)| {
                            let div = *s / r;
                            a[0] = div[0].norm();
                            a[1] = div[1].norm();
                            a[2] = div[2].norm();
                            a[3] = div[3].norm();
                            p[0] = div[0].arg().to_degrees();
                            p[1] = div[1].arg().to_degrees();
                            p[2] = div[2].arg().to_degrees();
                            p[3] = div[3].arg().to_degrees();
                        });
                });

            let (min_amp, max_amp) = match (min_amp, max_amp) {
                (Some(user_min), Some(user_max)) => (user_min, user_max),
                _ => {
                    // We need to work out the min and max ourselves.
                    let (data_min, data_max) = amps.iter().flatten().filter(|a| !a.is_nan()).fold(
                        (f64::INFINITY, 0.0),
                        |(acc_min, acc_max), &a| {
                            let acc_min = if a < acc_min { a } else { acc_min };
                            let acc_max = if a > acc_max { a } else { acc_max };
                            (acc_min, acc_max)
                        },
                    );

                    // Check any user-specified limits. Are they sensible relative
                    // to the data?
                    let min_amp = match min_amp {
                        Some(user_min_amp) => {
                            if user_min_amp > data_max {
                                warn!("User-specified plot minimum {user_min_amp} is larger than all data; ignoring");
                                data_min
                            } else {
                                user_min_amp
                            }
                        }
                        None => data_min,
                    };
                    let max_amp = match max_amp {
                        Some(user_max_amp) => {
                            if user_max_amp < data_min {
                                warn!("User-specified plot maximum {user_max_amp} is smaller than all data; ignoring");
                                data_max
                            } else {
                                user_max_amp
                            }
                        }
                        None => data_max,
                    };

                    // Failing all else, make sure the limits are sensible.
                    let min_amp = if min_amp.is_infinite() { 0.0 } else { min_amp };
                    let max_amp = if max_amp.abs() < f64::EPSILON {
                        1.0
                    } else {
                        max_amp
                    };

                    (min_amp, max_amp)
                }
            };

            let amps_ticks = Ticks::new(&font, min_amp, max_amp, 20);
            let phases_ticks = Ticks::new(&font, -180.0, 180.0, 45);
            let mut amps_plot = Plot::new(
                &format!("Amps for {obs_name}"),
                &meta_str,
                &font,
                AMPS_MARGIN,
                ignore_cross_pols,
            );
            let mut phases_plot = Plot::new(
                &format!("Phases for {obs_name}"),
                &meta_str,
                &font,
                0,
                ignore_cross_pols,
            );
            let amps_cells = amps_plot.cells(num_rows, num_cols);
            let phases_cells = phases_plot.cells(num_rows, num_cols);
            let num_plotted = total_num_tiles.min(num_rows * num_cols);
            for (i_tile, (amps, phases)) in amps
                .outer_iter()
                .zip(phases.outer_iter())
                .take(num_plotted)
                .enumerate()
            {
                let tile_name = match tile_names {
                    Some(names) => format!("{}: {}", i_tile, names[i_tile]),
                    None => format!("{i_tile}"),
                };
                let first_column = i_tile % num_cols == 0;
                amps_plot.tile(
                    amps_cells[i_tile],
                    first_column,
                    amps,
                    &amps_ticks,
                    &tile_name,
                    tile_name_font_size,
                );
                phases_plot.tile(
                    phases_cells[i_tile],
                    first_column,
                    phases,
                    &phases_ticks,
                    &tile_name,
                    tile_name_font_size,
                );
            }
            let amps_fig = amps_plot.into_figure();
            let phases_fig = phases_plot.into_figure();
            amps_fig.save_png(&output_amps)?;
            phases_fig.save_png(&output_phases)?;
            output_filenames.push(output_amps);
            output_filenames.push(output_phases);
        }

        Ok(output_filenames)
    }

    /// The pixel rectangle of one tile: `x1` and `y1` are exclusive.
    #[derive(Clone, Copy)]
    pub(super) struct Cell {
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
    }

    /// Split `total` pixels from `start` into `n` runs, giving the first few
    /// runs the leftover pixels.
    fn split_evenly(start: i32, total: i32, n: usize) -> Vec<(i32, i32)> {
        let n = n as i32;
        let (size, extra) = (total / n, total % n);
        let mut pos = start;
        (0..n)
            .map(|i| {
                let end = pos + size + i32::from(i < extra);
                let run = (pos, end);
                pos = end;
                run
            })
            .collect()
    }

    /// Round-number tick positions between `min` and `max`, using at most
    /// `max_points` of them, and no step smaller than `min_step`.
    fn key_points(min: f64, max: f64, max_points: usize, min_step: f64) -> (Vec<f64>, f64) {
        let (lo, hi) = (min.min(max), min.max(max));
        if lo == hi {
            return (vec![lo], 1.0);
        }
        let span = hi - lo;
        let mut scale = 10.0_f64.powf(span.log10().floor());
        if 1 + (span / scale).floor() as usize > max_points {
            scale *= 10.0;
        }
        'refine: loop {
            let coarse = scale;
            for divisor in [2.0, 5.0, 10.0] {
                let step = coarse / divisor;
                let first = lo + step - lo.rem_euclid(step);
                let last = hi - hi.rem_euclid(step);
                let num_points = 1 + ((last - first) / coarse * divisor) as usize;
                if step < min_step || num_points > max_points {
                    break 'refine;
                }
                scale = step;
            }
            scale = coarse / 10.0;
        }
        let scale = scale.max(min_step);
        let first = (lo / scale).ceil() as i64;
        let last = (hi / scale).floor() as i64;
        ((first..=last).map(|i| i as f64 * scale).collect(), scale)
    }

    /// Tick positions with their labels, as the y axis of every tile shows.
    pub(super) struct Ticks {
        min: f64,
        max: f64,
        ticks: Vec<(f64, String)>,
        label_width: f64,
        /// The least width, in pixels, of the area left of the first column.
        min_label_area: i32,
    }

    impl Ticks {
        fn new(font: &FontSource, min: f64, max: f64, min_label_area: i32) -> Ticks {
            let (points, scale) = key_points(min, max, 10, 0.0);
            let decimals = (-scale.log10()).ceil().max(1.0) as i32;
            let ticks: Vec<(f64, String)> = points
                .into_iter()
                .map(|p| {
                    let factor = 10.0_f64.powi(decimals);
                    let rounded = (p * factor).round() / factor;
                    (p, format!("{rounded:?}"))
                })
                .collect();
            let label_width = ticks
                .iter()
                .map(|(_, l)| layout_rich_text(font, l, TICK_LABEL_EM).width)
                .fold(0.0, f64::max);
            Ticks {
                min,
                max,
                ticks,
                label_width,
                min_label_area,
            }
        }
    }

    #[derive(Clone, Copy)]
    enum Align {
        Left,
        Centre,
        Right,
    }

    /// Filled shapes of one colour. Rectangles are given in pixels (with the
    /// origin at the top left and the end exclusive) so they are drawn crisply.
    #[derive(Default)]
    struct Layer {
        vertices: Vec<[f64; 2]>,
        codes: Vec<PathCode>,
    }

    impl Layer {
        fn rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
            // rizzma's y axis points up.
            let (x0, x1) = (f64::from(x0), f64::from(x1));
            let (y0, y1) = (f64::from(HEIGHT - y1), f64::from(HEIGHT - y0));
            self.vertices
                .extend([[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]]);
            self.codes.extend([
                PathCode::MoveTo,
                PathCode::LineTo,
                PathCode::LineTo,
                PathCode::LineTo,
                PathCode::ClosePoly,
            ]);
        }

        fn text(
            &mut self,
            font: &FontSource,
            text: &str,
            em: f64,
            x: f64,
            baseline: f64,
            align: Align,
        ) {
            let rich = layout_rich_text(font, text, em);
            let x = match align {
                Align::Left => x,
                Align::Centre => x - rich.width / 2.0,
                Align::Right => x - rich.width,
            };
            let shift = Affine2D::from_translation(x, f64::from(HEIGHT) - baseline);
            for path in &rich.paths {
                let path = path.transformed(&shift);
                match path.codes() {
                    Some(codes) => self.codes.extend_from_slice(codes),
                    None => self.codes.extend(
                        std::iter::once(PathCode::MoveTo)
                            .chain(std::iter::repeat(PathCode::LineTo))
                            .take(path.vertices().len()),
                    ),
                }
                self.vertices.extend_from_slice(path.vertices());
            }
        }

        fn into_patch(self, colour: Rgba, zorder: f64) -> Option<Patch> {
            if self.vertices.is_empty() {
                return None;
            }
            Some(
                Patch::new(Path::new(self.vertices, Some(self.codes)))
                    .facecolor(Some(colour))
                    .edgecolor(None)
                    .with_zorder(zorder),
            )
        }
    }

    /// A whole plot, built up in layers that are drawn from the bottom up.
    pub(super) struct Plot<'a> {
        font: &'a FontSource,
        x_offset: i32,
        ignore_cross_pols: bool,
        grid: [Layer; 2],
        flagged: Layer,
        markers: [Layer; 4],
        ink: Layer,
        legend: [Layer; 4],
    }

    impl<'a> Plot<'a> {
        /// Start a plot with the title, the timeblock metadata in the top left
        /// and the polarisation colour key in the top right. The tiles start
        /// `x_offset` pixels from the left.
        fn new(
            title: &str,
            meta: &str,
            font: &'a FontSource,
            x_offset: i32,
            ignore_cross_pols: bool,
        ) -> Plot<'a> {
            let mut plot = Plot {
                font,
                x_offset,
                ignore_cross_pols,
                grid: Default::default(),
                flagged: Layer::default(),
                markers: Default::default(),
                ink: Layer::default(),
                legend: Default::default(),
            };
            let centre = f64::from(x_offset + (WIDTH - x_offset) / 2);
            plot.ink
                .text(font, title, 48.4, centre, 42.5, Align::Centre);
            plot.ink.text(font, meta, 30.66, 10.0, 34.0, Align::Left);
            for (i, (first, second, _)) in POLS.iter().enumerate() {
                let x = f64::from(WIDTH - 500 + 80 * i as i32);
                plot.legend[i].text(font, first, 40.0, x, 42.0, Align::Left);
                plot.legend[i].text(font, second, 28.0, x + 30.0, 52.0, Align::Left);
            }
            plot
        }

        /// The cells that the tiles occupy, in row-major order.
        fn cells(&self, num_rows: usize, num_cols: usize) -> Vec<Cell> {
            let rows = split_evenly(TITLE_STRIP, HEIGHT - TITLE_STRIP, num_rows);
            let cols = split_evenly(self.x_offset, WIDTH - self.x_offset, num_cols);
            rows.iter()
                .flat_map(|&(y0, y1)| cols.iter().map(move |&(x0, x1)| Cell { x0, y0, x1, y1 }))
                .collect()
        }

        /// Scatter one tile's four polarisations against channel, or grey the
        /// tile out if every channel is flagged. Only the first column of
        /// tiles has a y axis.
        fn tile(
            &mut self,
            cell: Cell,
            first_column: bool,
            values: ArrayView1<[f64; 4]>,
            y_ticks: &Ticks,
            name: &str,
            font_size: i32,
        ) {
            let font = self.font;
            let num_chans = values.len() as i32;
            let em = 0.8 * f64::from(font_size);
            // The title, centred over the whole cell.
            self.ink.text(
                font,
                name,
                em,
                f64::from(cell.x0 + cell.x1) / 2.0,
                f64::from(cell.y0) + 1.066 * em,
                Align::Centre,
            );

            // The axes sit on the top and left of the plotting area.
            let y_axis = cell.y0 + (1.2 * f64::from(font_size)) as i32 + X_LABEL_AREA;
            let y_label_width = if first_column {
                (y_ticks.label_width.ceil() as i32 + TICK_LENGTH + Y_LABEL_GAP + 2)
                    .max(y_ticks.min_label_area)
            } else {
                0
            };
            let (left, right) = (cell.x0 + y_label_width, cell.x1);
            let (top, bottom) = (y_axis + 1, cell.y1);
            self.ink.rect(left, y_axis, right, y_axis + 1);
            if first_column {
                self.ink.rect(left - 1, top, left, bottom);
            }

            // Plotters' coordinate mapping: truncate to a whole pixel.
            let width = f64::from(right - 1 - left);
            let height = f64::from(bottom - 1 - top);
            let map_x = |v: f64| left + (width * v / f64::from(num_chans) + 1e-3).floor() as i32;
            let map_y = |v: f64| {
                let frac = (v - y_ticks.min) / (y_ticks.max - y_ticks.min);
                bottom - 1 - (height * frac + 1e-3).floor() as i32
            };

            let (x_points, _) = key_points(0.0, f64::from(num_chans), 10, 1.0);
            for v in x_points {
                let x = map_x(v);
                self.grid[0].rect(x, top + 1, x + 1, bottom - 1);
                self.ink.rect(x, y_axis - TICK_LENGTH, x + 1, y_axis);
                self.ink.text(
                    font,
                    &format!("{v}"),
                    TICK_LABEL_EM,
                    f64::from(x),
                    f64::from(y_axis - TICK_LENGTH - X_LABEL_GAP),
                    Align::Centre,
                );
            }
            for (v, label) in &y_ticks.ticks {
                let y = map_y(*v);
                self.grid[1].rect(left, y, right, y + 1);
                if first_column {
                    self.ink.rect(left - 1 - TICK_LENGTH, y, left - 1, y + 1);
                    self.ink.text(
                        font,
                        label,
                        TICK_LABEL_EM,
                        f64::from(left - 1 - TICK_LENGTH - Y_LABEL_GAP),
                        f64::from(y) + 0.5 + 0.5 * 0.729 * TICK_LABEL_EM,
                        Align::Right,
                    );
                }
            }

            if values.iter().all(|v| v.iter().any(|f| f.is_nan())) {
                self.flagged.rect(left, top, right, bottom);
                return;
            }

            for pol_index in 0..4 {
                let cross_pol = [1, 2].contains(&pol_index);
                if cross_pol && self.ignore_cross_pols {
                    continue;
                }
                let mut pixels = std::collections::BTreeSet::new();
                for (i, v) in values.iter().enumerate() {
                    let y = v[pol_index];
                    if y.is_nan() {
                        continue;
                    }
                    let (x, y) = (map_x(i as f64), map_y(y));
                    // Gains are plus signs, leakages hollow diamonds.
                    let offsets: &[(i32, i32)] = if cross_pol {
                        &[(-1, 0), (1, 0), (0, -1), (0, 1)]
                    } else {
                        &[(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)]
                    };
                    pixels.extend(
                        offsets
                            .iter()
                            .map(|(dx, dy)| (x + dx, y + dy))
                            .filter(|&(x, y)| x >= left && x < right && y >= top && y < bottom),
                    );
                }
                for (x, y) in pixels {
                    self.markers[pol_index].rect(x, y, x + 1, y + 1);
                }
            }
        }

        fn into_figure(self) -> Figure {
            let mut fig =
                Figure::new(f64::from(WIDTH) / DPI, f64::from(HEIGHT) / DPI).with_dpi(DPI);
            let ax = fig.add_axes(0.0, 0.0, 1.0, 1.0);
            ax.set_axis_off()
                .set_xlim(0.0, f64::from(WIDTH))
                .set_ylim(0.0, f64::from(HEIGHT))
                .set_facecolor(Rgba::TRANSPARENT);
            let [grid_x, grid_y] = self.grid;
            let mut layers = vec![(grid_x, GRID), (grid_y, GRID), (self.flagged, FLAGGED)];
            for (i, layer) in self.markers.into_iter().enumerate() {
                if !(self.ignore_cross_pols && [1, 2].contains(&i)) {
                    layers.push((layer, POLS[i].2));
                }
            }
            layers.push((self.ink, Rgba::BLACK));
            for (i, layer) in self.legend.into_iter().enumerate() {
                if !(self.ignore_cross_pols && [1, 2].contains(&i)) {
                    layers.push((layer, POLS[i].2));
                }
            }
            for (z, (layer, colour)) in layers.into_iter().enumerate() {
                if let Some(patch) = layer.into_patch(colour, z as f64) {
                    ax.add_patch(patch);
                }
            }
            fig
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_key_points() {
            let (points, scale) = key_points(-180.0, 180.0, 10, 0.0);
            assert_eq!(points, [-150.0, -100.0, -50.0, 0.0, 50.0, 100.0, 150.0]);
            assert_eq!(scale, 50.0);

            let (points, _) = key_points(0.0, 32.0, 10, 1.0);
            assert_eq!(points, [0.0, 5.0, 10.0, 15.0, 20.0, 25.0, 30.0]);

            // Channels are whole numbers, so never get ticks between them.
            let (points, _) = key_points(0.0, 4.0, 10, 1.0);
            assert_eq!(points, [0.0, 1.0, 2.0, 3.0, 4.0]);

            let (points, scale) = key_points(0.0, 1.0, 10, 0.0);
            assert_eq!(points.len(), 11);
            assert_eq!(scale, 0.1);
        }

        #[test]
        fn test_tick_labels() {
            let font = FontSource::dejavu_sans();
            let labels = |min, max| {
                Ticks::new(&font, min, max, 0)
                    .ticks
                    .into_iter()
                    .map(|(_, l)| l)
                    .collect::<Vec<_>>()
            };
            assert_eq!(labels(-180.0, 180.0)[..2], ["-150.0", "-100.0"]);
            assert_eq!(labels(0.0, 1.0)[..3], ["0.0", "0.1", "0.2"]);
        }

        #[test]
        fn test_split_evenly() {
            // The first runs get the leftover pixels.
            let runs = split_evenly(58, 1742, 10);
            assert_eq!(runs[0], (58, 233));
            assert_eq!(runs[1], (233, 408));
            assert_eq!(runs[2], (408, 582));
            assert_eq!(runs[9].1, 1800);
        }
    }
}
