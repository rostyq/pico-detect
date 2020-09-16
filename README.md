# pico-detect

This library is a reimplementation of _Pixel Intensity Comparison-based Object_ (PICO) detection algorithms in Rust:

- `Detector`: Cascade of binary classifiers from [pico](https://github.com/nenadmarkus/pico);
- `Localizer`: Localization with an ensemble of randomized trees from [picojs](https://github.com/nenadmarkus/picojs) (see `lploc.js`);
- `Shaper`: Alignment with an ensemble of regression trees from [dlib](https://github.com/davisking/dlib) (see `shape_predictor`).

<!-- ![visualization example](tests/assets/Lenna_(image_result).png) -->

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

