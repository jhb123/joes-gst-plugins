use gst::glib;
use gst::prelude::*;

glib::wrapper! {
    pub struct PlaneExpand(ObjectSubclass<imp::PlaneExpand>) @extends gst_base::BaseTransform, gst::Element, gst::Object;
}

pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        "planeExpand",
        gst::Rank::NONE,
        PlaneExpand::static_type(),
    )
}

mod imp {

    use gst::glib;
    use gst_video::subclass::prelude::*;
    use gst_video::VideoFrameExt;

    use once_cell::sync::Lazy;

    #[allow(dead_code)]
    static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
        gst::DebugCategory::new(
            "planeExpand",
            gst::DebugColorFlags::empty(),
            Some("Rust Plane Expand"),
        )
    });


    #[allow(dead_code)]
    #[derive(Default)]
    pub struct PlaneExpand { }

    impl ObjectImpl for PlaneExpand {}

    impl GstObjectImpl for PlaneExpand {}

    #[glib::object_subclass]
    impl ObjectSubclass for PlaneExpand {
        const NAME: &'static str = "PlaneExpand";
        type Type = super::PlaneExpand;
        type ParentType = gst_video::VideoFilter;
    }

    impl BaseTransformImpl for PlaneExpand {
        const MODE: gst_base::subclass::BaseTransformMode =
            gst_base::subclass::BaseTransformMode::NeverInPlace;
        const PASSTHROUGH_ON_SAME_CAPS: bool = false;
        const TRANSFORM_IP_ON_PASSTHROUGH: bool = true;

        fn transform_caps(
            &self,
            direction: gst::PadDirection,
            caps: &gst::Caps,
            filter: Option<&gst::Caps>,
        ) -> Option<gst::Caps> {
    
            let other_caps = if direction == gst::PadDirection::Sink {
                let mut caps = caps.clone();
                for s in caps.make_mut().iter_mut() {
                    if let Ok(w) = s.get::<i32>("width") {
                        s.set("width", w * 3);
                    }
                }
                caps
            } else {
                let mut caps = caps.clone();
                for s in caps.make_mut().iter_mut() {
                    if let Ok(w) = s.get::<i32>("width") {
                        s.set("width", w / 3);
                    }
                }
                caps
            };    
            if let Some(filter) = filter {
                Some(filter.intersect_with_mode(&other_caps, gst::CapsIntersectMode::First))
            } else {
                Some(other_caps)
            }    
        }    
    }

    impl ElementImpl for PlaneExpand {
        fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
            static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
                gst::subclass::ElementMetadata::new(
                    "View Planes Tool",
                    "Filter/Effect/Converter/Video",
                    "View the raw planes side by side.",
                    "Joseph Briggs",
                )
            });

            Some(&*ELEMENT_METADATA)
        }

        fn pad_templates() -> &'static [gst::PadTemplate] {
            static PAD_TEMPLATES: Lazy<Vec<gst::PadTemplate>> = Lazy::new(|| {
                let caps = gst_video::VideoCapsBuilder::new()
                    .format_list([gst_video::VideoFormat::I420])
                    .build();
                let src_pad_template = gst::PadTemplate::new(
                    "src",
                    gst::PadDirection::Src,
                    gst::PadPresence::Always,
                    &caps,
                )
                .unwrap();

                let caps = gst_video::VideoCapsBuilder::new()
                    .format_list([gst_video::VideoFormat::I420])
                    .build();
                let sink_pad_template = gst::PadTemplate::new(
                    "sink",
                    gst::PadDirection::Sink,
                    gst::PadPresence::Always,
                    &caps,
                )
                .unwrap();

                vec![src_pad_template, sink_pad_template]
            });

            PAD_TEMPLATES.as_ref()
        }
    }

    impl PlaneExpand {}

    impl VideoFilterImpl for PlaneExpand {

        fn transform_frame(
            &self,
            in_frame: &gst_video::VideoFrameRef<&gst::BufferRef>,
            out_frame: &mut gst_video::VideoFrameRef<&mut gst::BufferRef>,
        ) -> Result<gst::FlowSuccess, gst::FlowError> {

            assert_eq!(in_frame.width()*3, out_frame.width());

            let in_plane_stride = in_frame.plane_stride();
            let out_plane_stride = out_frame.plane_stride().to_vec();

            for plane in 0..in_frame.n_planes() {

                let offset = match plane  {
                    0 => 0,
                    1 => in_frame.comp_width(1).try_into().unwrap(),
                    2 => (in_frame.comp_width(1)+in_frame.comp_width(2)).try_into().unwrap(),
                    _ => unreachable!()
                };

                let in_plane = in_frame.plane_data(plane).unwrap();
                let in_lines = in_plane.chunks_exact(in_plane_stride[plane as usize] as usize);

                let out_plane = out_frame.plane_data_mut(plane).unwrap();
                let out_lines = out_plane.chunks_exact_mut(out_plane_stride[plane as usize] as usize);

                for (in_line, out_line) in std::iter::zip(in_lines,out_lines) {
                    out_line.fill(125);
                    out_line[offset..in_line.len()+offset].copy_from_slice(in_line);
                }
            }
            Ok(gst::FlowSuccess::Ok)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use test::Bencher;

        fn setup_benchmarks(width: u32, height: u32, b: &mut Bencher) {
            let _ = gst::init();
           
            let plugin = PlaneExpand {};

            let info = gst_video::VideoInfo::builder(gst_video::VideoFormat::I420, width, height)
                .build()
                .unwrap();
            let data_inframe = vec![0; (width * height * 3) as usize];
            let buffer_inframe = gst::Buffer::from_slice(data_inframe);
            let inframe = test::black_box(gst_video::VideoFrame::from_buffer_readable(buffer_inframe, &info).unwrap());

            let info = gst_video::VideoInfo::builder(gst_video::VideoFormat::I420, width*3, height)
                .build()
                .unwrap();
            let data_outframe = vec![0; (width * 3 * height * 3) as usize];
            let buffer_outframe = gst::Buffer::from_slice(data_outframe);
            let mut outframe = test::black_box(gst_video::VideoFrame::from_buffer_writable(buffer_outframe, &info).unwrap());

            b.iter(|| {
                plugin.transform_frame(
                    &inframe.as_video_frame_ref(),
                    &mut outframe.as_mut_video_frame_ref(),
                )
            });
        }

        #[bench]
        fn bench_plugin_4k(b: &mut Bencher) {
            setup_benchmarks(4096, 2160, b);
        }

        #[bench]
        fn bench_plugin_1080p(b: &mut Bencher) {
            setup_benchmarks(1920, 1080, b);
        }

        #[bench]
        fn bench_plugin_720p(b: &mut Bencher) {
            setup_benchmarks(1280, 720, b);
        }

    }
}

