# Overview
This is a collection of GStreamer plugins for debugging and testing tools that analyse the quality of a stream.
- `yuvOffset` is for applying an offset to the each of the YUV planes.

## Building


## Testing
```
export GST_DEBUG=0,videotestsrc:6
gst-launch-1.0 videotestsrc ! yuvOffset ! videoconvert ! autovideosink
```