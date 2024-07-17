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
    use gst::State;
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

    const DEFAULT_SHIFT: u8 = 0;

    #[derive(Debug, Clone, Copy)]
    struct Settings {
        y: u8,
        u: u8,
        v: u8
    }

    impl Default for Settings {
        fn default() -> Self {
            Self { y: DEFAULT_SHIFT, u: DEFAULT_SHIFT, v: DEFAULT_SHIFT }
        }
    }

    #[derive(Default)]
    pub struct YuvOffset {
        settings: Mutex<Settings>,
        state: Mutex<Option<State>>,
    }

    impl ObjectImpl for YuvOffset {
        fn properties() -> &'static [glib::ParamSpec] {

            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecUChar::builder("y")
                        .nick("y")
                        .blurb("Offset applied to the Y plane")
                        .default_value(DEFAULT_SHIFT)
                        .mutable_playing()
                        .build(),
                        glib::ParamSpecUChar::builder("u")
                        .nick("u")
                        .blurb("Offset applied to the U plane")
                        .default_value(DEFAULT_SHIFT)
                        .mutable_playing()
                        .build(),
                        glib::ParamSpecUChar::builder("v")
                        .nick("v")
                        .blurb("Offset applied to the V plane")
                        .default_value(DEFAULT_SHIFT)
                        .mutable_playing()
                        .build(),
                ]
            });

            PROPERTIES.as_ref()
        }
        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "y" => {
                    let mut settings = self.settings.lock().unwrap();
                    let y = value.get().expect("type checked upstream");
                    settings.y = y;
                }
                "u" => {
                    let mut settings = self.settings.lock().unwrap();
                    let u = value.get().expect("type checked upstream");
                    settings.u = u;
                }
                "v" => {
                    let mut settings = self.settings.lock().unwrap();
                    let v = value.get().expect("type checked upstream");
                    settings.v = v;
                }
                _ => unimplemented!()
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "y" => {
                    let settings = self.settings.lock().unwrap();
                    settings.y.to_value()
                }
                "u" => {
                    let settings = self.settings.lock().unwrap();
                    settings.u.to_value()
                }
                "v" => {
                    let settings = self.settings.lock().unwrap();
                    settings.v.to_value()
                }
                _ => unimplemented!()
            }
        }
    

    }


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
            let settings = *self.settings.lock().unwrap();

            for plane in 0..in_frame.n_planes(){
                let offset = match plane {
                    0 => settings.y,
                    1 => settings.u,
                    2 => settings.v,
                    _ => unimplemented!()
                };
                let in_plane = in_frame.plane_data(plane).unwrap();
                let out_plane = out_frame.plane_data_mut(plane).unwrap();
                out_plane.iter_mut().enumerate().for_each(|(i,x)| {*x=in_plane[i].wrapping_add(offset);})
            }            
            Ok(gst::FlowSuccess::Ok)
        }
    }
    
}