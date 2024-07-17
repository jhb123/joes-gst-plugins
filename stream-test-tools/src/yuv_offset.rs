use gst::glib;
use gst::prelude::*;

glib::wrapper! {
    pub struct YuvOffset(ObjectSubclass<imp::YuvOffset>) @extends gst_base::BaseTransform, gst::Element, gst::Object;
}

pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        "yuvOffset",
        gst::Rank::NONE,
        YuvOffset::static_type(),
    )
}

mod imp {
    use gst::glib;
    use gst::prelude::*;
    use gst_video::subclass::prelude::*;
    use gst_video::VideoFrameExt;

    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
        gst::DebugCategory::new(
            "yuvOffset",
            gst::DebugColorFlags::empty(),
            Some("Rust YUV Offset"),
        )
    });

    #[derive(Default)]
    pub struct YuvOffset {}
    impl ObjectImpl for YuvOffset {}
    impl GstObjectImpl for YuvOffset {}

    #[glib::object_subclass]
    impl ObjectSubclass for YuvOffset {
        const NAME: &'static str = "yuvOffset";
        type Type = super::YuvOffset;
        type ParentType = gst_video::VideoFilter;
    }


    impl BaseTransformImpl for YuvOffset {
        const MODE: gst_base::subclass::BaseTransformMode = gst_base::subclass::BaseTransformMode::NeverInPlace;
        const PASSTHROUGH_ON_SAME_CAPS: bool = false;
        const TRANSFORM_IP_ON_PASSTHROUGH: bool = true;
        fn transform_caps(
                &self,
                direction: gst::PadDirection,
                caps: &gst::Caps,
                filter: Option<&gst::Caps>,
            ) -> Option<gst::Caps> {
                gst::debug!(
                    CAT,
                    imp = self,
                    "tranforming caps {}",
                    caps
                );
                Some(caps.clone())
        }
    }

    impl ElementImpl for YuvOffset {
        fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
            static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
                gst::subclass::ElementMetadata::new(
                    "YUV Offset Tool",
                    "Filter/Effect/Converter/Video",
                    "Modify the incoming YUV planes to test tools for analysing videos.",
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
                    &caps).unwrap();

                let caps = gst_video::VideoCapsBuilder::new()
                    .format_list([gst_video::VideoFormat::I420])
                    .build();
                let sink_pad_template = gst::PadTemplate::new(
                    "sink", 
                    gst::PadDirection::Sink, 
                    gst::PadPresence::Always, 
                    &caps).unwrap();
    
                

                vec![src_pad_template, sink_pad_template]
            });

            PAD_TEMPLATES.as_ref()
        }
    }

    impl YuvOffset {

    }

    impl VideoFilterImpl for YuvOffset {
        fn transform_frame(
            &self,
            in_frame: &gst_video::VideoFrameRef<&gst::BufferRef>,
            out_frame: &mut gst_video::VideoFrameRef<&mut gst::BufferRef>,
        ) -> Result <gst::FlowSuccess, gst::FlowError> {
            for plane in 0..in_frame.n_planes(){
                let in_plane = in_frame.plane_data(plane).unwrap();
                let out_plane = out_frame.plane_data_mut(plane).unwrap();
                out_plane.iter_mut().enumerate().for_each(|(i,x)| {*x=in_plane[i].wrapping_add(200);})
            }            
            Ok(gst::FlowSuccess::Ok)
        }
    }
    
}