# HTML Image Inliner

HTML Image Inliner is a command-line tool written in Rust that inlines local images into HTML files using base64
encoding.

## Features

- **Inlines local images** into HTML files using base64 encoding.
- Supports **parallel processing** of images for improved performance.

## Performance

In tests, this program has shown to be faster than similar tools. For example, it can inline 32 images (each image is
about 300KiB) into a source HTML file of about 9KiB, resulting in an inlined HTML file of about 11MiB, in approximately
187ms.

## Usage

```shell
html-images-inliner <input_1.html> <input_2.html> ...
```

The output files with inlined images are saved in the same directory as the input files, with the
extension `.inlined.html`.

## Alternatives

There are other similar tools that provide more features:

- [inliners](https://github.com/makovich/inliners)
- [monolith](https://github.com/Y2Z/monolith)