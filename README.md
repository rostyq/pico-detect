[![crates-badge]][crates]
[![docs-badge]][docs]
![license-badge]

# pico-detect

This library is a reimplementation of _Pixel Intensity Comparison-based Object_ (PICO) detection algorithms in Rust:

- `Detector`: Cascade of binary classifiers from [pico];
- `Localizer`: Localization with an ensemble of randomized trees from [picojs](https://github.com/nenadmarkus/picojs) (see `lploc.js`);
- `Shaper`: Alignment with an ensemble of regression trees from [dlib](https://github.com/davisking/dlib) (see `shape_predictor`).

## Example

To run CLI example, which takes an image, finds all faces, detects some landmarks and pupils:

> **NOTE**: [Git LFS](https://git-lfs.github.com/) is needed to resolve binary files with `git clone`.
>
> If you don't want to use Git LFS you can download models (and test image) direct from this repo
> (see **model** column in the table below)
> and put them under [`models/`](./models) directory.

```sh
cargo run --release --example detect-faces -- --models-dir models -i "assets/test.png" --score 35.0 -o result.png
```

Output image `result.png` should be like this:

![visualization example](./assets/result.png)

## Models

Each algorithm requires to be loaded with correspondent binary model.

| model                     | algorithm   | source                             | Description               |
|---------------------------|-------------|------------------------------------|---------------------------|
| [face.detector.bin]       | `Detector`  | [pico]                             | Human face classifier     |
| [pupil.localizer.bin]     | `Localizer` | [puploc]                           | Human eye pupil localizer |
| [face-5.shaper.bin]       | `Shaper`    | [shape_predictor_5_face_landmarks] | Human 5 face landmarks    |

## References

1. [N. Markus, M. Frljak, I. S. Pandzic, J. Ahlberg and R. Forchheimer, "Object Detection with Pixel Intensity Comparisons Organized in Decision Trees"](http://arxiv.org/abs/1305.4537)

2. [Eye pupil localization with an ensemble of randomized trees](https://across.fer.hr/_download/repository/PR4885.pdf)

3. [One Millisecond Face Alignment with an Ensemble of Regression Trees](https://www.cv-foundation.org/openaccess/content_cvpr_2014/papers/Kazemi_One_Millisecond_Face_2014_CVPR_paper.pdf)

[crates]: https://crates.io/crates/pico-detect
[docs]: https://docs.rs/pico-detect
[docs-badge]: https://docs.rs/pico-detect/badge.svg
[crates-badge]: https://img.shields.io/crates/v/pico-detect
[license-badge]: https://img.shields.io/crates/l/pico-detect

[pico]: https://github.com/nenadmarkus/pico

[face.detector.bin]: https://github.com/rostyq/pico-detect/raw/master/models/face.detector.bin
[pupil.localizer.bin]: https://github.com/rostyq/pico-detect/raw/master/models/pupil.localizer.bin
[face-5.shaper.bin]: https://github.com/rostyq/pico-detect/raw/master/models/face-5.shaper.bin

[puploc]: https://drone.nenadmarkus.com/data/blog-stuff/puploc.bin
[shape_predictor_5_face_landmarks]: https://github.com/davisking/dlib-models#shape_predictor_5_face_landmarksdatbz2
