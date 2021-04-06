# ciya-rs

Ciyaify your image.

![from](examples/original.png)
![to](examples/result.png)

- `ciya-cli` - a command-line tool that ciyaify specified images.
- `ciya-bot` - a telegram bot that ciyaify given images or stickers. [@ciyaify_bot](https://t.me/ciyaify_bot)

## Get Started

> Currently only Linux is supported.

### Linux

- Install OpenCV library. Make sure to install `-dev` packages if your distribution provides.

> For Archlinux users:
> 
> Install OpenCV 4.5.1, not the latest version which is unsupported.

- ``` make all ```
- Built binaries are located in `dist` directory.

## Todo

- [ ] `detectors::StandardDetector`
- [ ] configurable bot settings
- [ ] release (deal with onnxruntime)
- [ ] add a proper license