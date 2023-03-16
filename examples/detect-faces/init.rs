use anyhow::{anyhow, Context, Result};
use image::DynamicImage;
use pico_detect::{Detector, Localizer, MultiscaleDetector, PerturbatingLocalizer, Shaper};

use crate::args::Args;

pub fn initialize(args: &Args, image: &DynamicImage) -> Result<(MultiscaleDetector, Shaper, PerturbatingLocalizer)> {
    let detector = detector!(args);
    let shaper = shaper!(args);
    let localizer = localizer!(args);

    let min_size = args.min_size;
    let max_size = args
        .max_size
        .unwrap_or_else(|| image.height().min(image.width()));
    let scale_factor = args.scale_factor;
    let shift_factor = args.shift_factor;

    let top_padding = args.top_padding;
    let left_padding = args.left_padding;
    let right_padding = args.right_padding;
    let bottom_padding = args.bottom_padding;

    let intersection_threshold = args.intersection_threshold;
    let score_threshold = args.score_threshold;

    if max_size < min_size {
        return Err(anyhow!("min size should be less than max size"));
    }

    let detector = MultiscaleDetector::builder()
        .map_multiscale_builder(|b| {
            b.with_min_size(min_size)
                .with_max_size(max_size)
                .with_scale_factor(scale_factor)
                .with_shift_factor(shift_factor)
                .map_padding(|b| {
                    b.with_top(top_padding)
                        .with_left(left_padding)
                        .with_bottom(bottom_padding)
                        .with_right(right_padding)
                })
        })
        .map_clusterizer_builder(|b| {
            b.with_intersection_threshold(intersection_threshold)
                .with_score_threshold(score_threshold)
        })
        .build(detector)
        .map_err(|e| anyhow!(e))?;

    let nperturbs = args.localizer_perturbs;
    let localizer = PerturbatingLocalizer::builder()
        .with_perturbs(nperturbs)
        .build(localizer)
        .map_err(|e| anyhow!(e))?;

    Ok((detector, shaper, localizer))
}
