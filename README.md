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

```sh
cargo run --release --example cli -- --input tests/assets/Lenna_\(test_image\).png --output result.png
```

Output image `result.png` should be like this:

![visualization example](./tests/assets/Lenna_(result_image).png)

## Models

Each algorithm requires to be loaded with correspondent binary model.

| model                     | algorithm   | source                             | Description               |
|---------------------------|-------------|------------------------------------|---------------------------|
| [facefinder]              | `Detector`  | [pico]                             | Human face classifier     |
| [puploc]                  | `Localizer` | [puploc source]                    | Human eye pupil localizer |
| [shaper_5_face_landmarks] | `Shaper`    | [shape_predictor_5_face_landmarks] | Human 5 face landmarks    |

## Roadmap

* [x] object detection;
* [x] object localization;
* [x] shape prediction;
* [x] cli example;
* [ ] WebAssembly support (`wasm32-unknown-unknown` target) and web example.

## References

1. [N. Markus, M. Frljak, I. S. Pandzic, J. Ahlberg and R. Forchheimer, "Object Detection with Pixel Intensity Comparisons Organized in Decision Trees"](http://arxiv.org/abs/1305.4537)

2. [Eye pupil localization with an ensemble of randomized trees](https://across.fer.hr/_download/repository/PR4885.pdf)

3. [One Millisecond Face Alignment with an Ensemble of Regression Trees](https://www.cv-foundation.org/openaccess/content_cvpr_2014/papers/Kazemi_One_Millisecond_Face_2014_CVPR_paper.pdf)

[crates]: https://crates.io/crates/pico-detect
[docs]: https://docs.rs/pico-detect/0.2.0/pico_detect
[docs-badge]: https://docs.rs/pico-detect/badge.svg
[crates-badge]: https://img.shields.io/crates/v/pico-detect
[license-badge]: https://img.shields.io/crates/l/pico-detect

[pico]: https://github.com/nenadmarkus/pico

[facefinder]: https://github.com/rostyslavb/pico-detect/raw/feature/shaper/models/facefinder
[puploc]: https://github.com/rostyslavb/pico-detect/raw/feature/shaper/models/puploc.bin
[shaper_5_face_landmarks]: https://github.com/rostyslavb/pico-detect/raw/feature/shaper/models/shaper_5_face_landmarks.bin

[puploc source]: https://drone.nenadmarkus.com/data/blog-stuff/puploc.bin
[shape_predictor_5_face_landmarks]: https://github.com/davisking/dlib-models#shape_predictor_5_face_landmarksdatbz2
