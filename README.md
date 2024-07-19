# Overview
This is a collection of GStreamer plugins made with Rust. The project is split into subpackages.

## stream-test-tools 
  This set of plugins is for debugging and testing other tools that analyse the quality of a stream. For example, you may want to test your PSNR analysis with yuOffset.
 - `yuvOffset` is for applying an offset to the each of the YUV planes.

# Building
First ensure that you have downloaded the latest version of the gstreamer library. On Mac, you can follow the instructions on the [official GStreamer site](https://gstreamer.freedesktop.org/documentation/installing/on-mac-osx.html?gi-language=c#download-and-install-the-sdk).

Set the environment variable
```
export PKG_CONFIG_PATH=/Library/Frameworks/GStreamer.framework/Versions/1.0/lib/pkgconfig/
```
and run 
```
cargo build --release
```
to build all of the plugins. The plugins will be found in `./target/release`. set the `GST_PLUGIN_PATH` variable
```
export GST_PLUGIN_PATH=./target/release
```
and inspect the plugin with e.g.
```
gst-inspect-1.0 yuvOffset
```
Use the plugin in a pipeline like this
```
gst-launch-1.0 videotestsrc ! video/x-raw,width=640,height=480,format=I420 ! yuvOffset ! videoconvert ! autovideosink
```
# Testing
Set the environment variable
```
export DYLD_LIBRARY_PATH=/Library/Frameworks/GStreamer.framework/Versions/1.0/lib:$DYLD_LIBRARY_PATH
```
and run either 
```
cargo bench
```
or
```
cargo test
```

Test the plugin and monitor the stack traces.
```
export GST_DEBUG=0
gst-launch-1.0 videotestsrc ! video/x-raw,width=640,height=480,format=I420 ! yuvOffset ! videoconvert ! autovideosink
export PID=$(pgrep gst-launch-1.0)
sudo dtrace -p $PID -n 'profile-997 /pid == '$PID'/ { @[ustack(100)] = count(); }' -o out.stacks
```

## Current benchmarks
The `transform_frame` function is the only function which is not, essentially, boiler plate. This is the function which is bench marked. For reference, a 60fps source corresponds to 16,666,666 ns.
```
test yuv_offset::imp::tests::bench_plugin_1080p ... bench:      68,172.91 ns/iter (+/- 4,403.17)
test yuv_offset::imp::tests::bench_plugin_4k    ... bench:     416,321.35 ns/iter (+/- 65,630.60)
test yuv_offset::imp::tests::bench_plugin_720p  ... bench:      25,954.32 ns/iter (+/- 7,150.43)
```