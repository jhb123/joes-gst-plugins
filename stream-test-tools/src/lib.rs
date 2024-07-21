#![feature(test)]
#![feature(try_find)]
#![feature(iter_advance_by)]
extern crate test;

use gst::glib;

mod yuv_offset;
mod plane_expand;

fn plugin_init(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    yuv_offset::register(plugin)?;
    plane_expand::register(plugin)?;
    Ok(())
}

gst::plugin_define!(
    stream_test_tools,
    env!("CARGO_PKG_DESCRIPTION"),
    plugin_init,
    concat!(env!("CARGO_PKG_VERSION"), "-", env!("COMMIT_ID")),
    "MIT/X11",
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_REPOSITORY"),
    env!("BUILD_REL_DATE")
);

