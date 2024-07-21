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
    use gst_video::{VideoFormat, VideoFrameExt};

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
                    .format_list([gst_video::VideoFormat::I420, VideoFormat::Rgb])
                    .build();
                let src_pad_template = gst::PadTemplate::new(
                    "src",
                    gst::PadDirection::Src,
                    gst::PadPresence::Always,
                    &caps,
                )
                .unwrap();

                let caps = gst_video::VideoCapsBuilder::new()
                    .format_list([gst_video::VideoFormat::I420, VideoFormat::Rgb])
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

    impl PlaneExpand {

    }

    impl VideoFilterImpl for PlaneExpand {

        fn transform_frame(
            &self,
            in_frame: &gst_video::VideoFrameRef<&gst::BufferRef>,
            out_frame: &mut gst_video::VideoFrameRef<&mut gst::BufferRef>,
        ) -> Result<gst::FlowSuccess, gst::FlowError> {

            assert_eq!(in_frame.width()*3, out_frame.width());

            match in_frame.n_planes() {
                1 => single_plane_split(in_frame, out_frame),
                3 => three_plane_split(in_frame, out_frame),
                4 => todo!() ,
                _ => unreachable!()
            }
            
        }
    }

    fn single_plane_split(in_frame: &gst_video::VideoFrameRef<&gst::BufferRef>, out_frame: &mut gst_video::VideoFrameRef<&mut gst::BufferRef>) -> Result<gst::FlowSuccess, gst::FlowError> { 

        let in_width = in_frame.width() as usize;
        let out_width = out_frame.width() as usize;
        let in_stride = in_frame.plane_stride()[0] as usize;
        let in_data = in_frame.plane_data(0).unwrap();
        let out_stride = out_frame.plane_stride()[0] as usize;
        let out_data = out_frame.plane_data_mut(0).unwrap();

        let in_line_bytes = in_width * 3;
        let out_line_bytes = out_width * 3;
        assert!(in_line_bytes <= in_stride);
        assert!(out_line_bytes <= out_stride);

        for (in_line, out_line) in in_data
                .chunks_exact(in_stride)
                .zip(out_data.chunks_exact_mut(out_stride))
            {
                out_line.fill(0);
                for (pix_idx, in_p) in in_line[..in_line_bytes].chunks_exact(3).enumerate() {
                    out_line[3*(pix_idx)] = in_p[0];
                    out_line[3*in_width+3*(pix_idx)+1] = in_p[1];
                    out_line[6*in_width+3*(pix_idx)+2] = in_p[2];
                }
        }
        Ok(gst::FlowSuccess::Ok)
    }

    fn three_plane_split(in_frame: &gst_video::VideoFrameRef<&gst::BufferRef>, out_frame: &mut gst_video::VideoFrameRef<&mut gst::BufferRef>) -> Result<gst::FlowSuccess, gst::FlowError> {
        let in_plane_stride = in_frame.plane_stride();
        let out_plane_stride = out_frame.plane_stride().to_vec();
    
        for plane in 0..3 {
    
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
    
    #[cfg(test)]
    mod tests {
        use super::*;
        use test::Bencher;

        #[test]
        fn test_rgb_transform_red(){
            let data_inframe = [255,0,0,0,255,0,0,0];
            let outframe = run_plugin_rgb_tests(data_inframe);
            assert_eq!(outframe.plane_data(0).unwrap(),[255,0,0,0, 0,0,0,0, 0,0,0,0, 255,0,0,0, 0,0,0,0, 0,0,0,0,]);
        }

        #[test]
        fn test_rgb_transform_green(){
            let data_inframe = [0,255,0,0,0,255,0,0];
            let outframe = run_plugin_rgb_tests(data_inframe);
            assert_eq!(outframe.plane_data(0).unwrap(),[0,0,0,0, 255,0 ,0,0, 0,0,0,0, 0,0,0,0, 255,0,0,0, 0,0,0,0,]);
        }

        #[test]
        fn test_rgb_transform_blue(){
            let data_inframe = [0,0,255,0,0,0,255,0];
            let outframe = run_plugin_rgb_tests(data_inframe);
            assert_eq!(outframe.plane_data(0).unwrap(),[0,0,0,0, 0,0,0,0, 255,0,0,0, 0,0,0,0, 0,0,0,0, 255, 0,0,0,]);
        }


        fn run_plugin_rgb_tests(data_inframe: [u8; 8]) -> gst_video::VideoFrame<gst_video::video_frame::Writable> {
            let _ = gst::init();
        
            const WIDTH: u32 = 1;
            const HEIGHT: u32 = 2;
        
            let plugin = PlaneExpand {};
        
            let info = gst_video::VideoInfo::builder(gst_video::VideoFormat::Rgb, WIDTH, HEIGHT)
                .build()
                .unwrap();
            let buffer_inframe = gst::Buffer::from_slice(data_inframe);
            let inframe = gst_video::VideoFrame::from_buffer_readable(buffer_inframe, &info).unwrap();
                
            let info = gst_video::VideoInfo::builder(gst_video::VideoFormat::Rgb, WIDTH*3, HEIGHT)
                .build()
                .unwrap();
            let data_outframe = [0; (WIDTH * HEIGHT * 4 * 3 ) as usize];
            let buffer_outframe = gst::Buffer::from_slice(data_outframe);
            let mut outframe = gst_video::VideoFrame::from_buffer_writable(buffer_outframe, &info).unwrap();
            let _ = plugin.transform_frame(
                &inframe.as_video_frame_ref(),
                &mut outframe.as_mut_video_frame_ref(),
            );
            outframe
        }
        
        fn setup_benchmarks_rgb(width: u32, height: u32, b: &mut Bencher) {
            let _ = gst::init();
           
            let plugin = PlaneExpand {};

            let info = gst_video::VideoInfo::builder(gst_video::VideoFormat::Rgb, width, height)
                .build()
                .unwrap();
            let data_inframe = vec![0; (width * height * 3) as usize];
            let buffer_inframe = gst::Buffer::from_slice(data_inframe);
            let inframe = test::black_box(gst_video::VideoFrame::from_buffer_readable(buffer_inframe, &info).unwrap());

            let info = gst_video::VideoInfo::builder(gst_video::VideoFormat::Rgb, width*3, height)
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


        fn setup_benchmarks_yuv(width: u32, height: u32, b: &mut Bencher) {
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
        fn bench_plugin_4k_yuv(b: &mut Bencher) {
            setup_benchmarks_yuv(4096, 2160, b);
        }

        #[bench]
        fn bench_plugin_1080p_yuv(b: &mut Bencher) {
            setup_benchmarks_yuv(1920, 1080, b);
        }

        #[bench]
        fn bench_plugin_720p_yuv(b: &mut Bencher) {
            setup_benchmarks_yuv(1280, 720, b);
        }

        #[bench]
        fn bench_plugin_4k_rgb(b: &mut Bencher) {
            setup_benchmarks_rgb(4096, 2160, b);
        }

        #[bench]
        fn bench_plugin_1080p_rgb(b: &mut Bencher) {
            setup_benchmarks_rgb(1920, 1080, b);
        }

        #[bench]
        fn bench_plugin_720p_rgb(b: &mut Bencher) {
            setup_benchmarks_rgb(1280, 720, b);
        }

    }
}

