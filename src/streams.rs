use gstreamer::{MessageView, prelude::*};

fn run_gstreamer_pipeline(rtsp_location: &str) {
    gstreamer::init().unwrap();

    let pipeline = gstreamer::Pipeline::default();
    let rtspsrc = gstreamer::ElementFactory::make("rtspsrc")
        .property("location", rtsp_location)
        .build().unwrap();
    let appsink = gstreamer_app::AppSink::builder().build();

    appsink.set_callbacks(
        gstreamer_app::AppSinkCallbacks::builder()
        .new_sample(|appsink| {
            appsink.pull_sample().map_err(|_| gstreamer::FlowError::Eos).unwrap();
            println!("Pulled sample");

            Ok(gstreamer::FlowSuccess::Ok)
        }).build(),
    );

    pipeline.add_many([&rtspsrc, appsink.upcast_ref()]).unwrap();
    rtspsrc.link(&appsink).unwrap();

    pipeline.set_state(gstreamer::State::Playing).unwrap();

    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed(None) {
        match msg.view() {
            MessageView::Eos(_) => break,
            MessageView::Error(err) => {
                println!("Error: {:?}", err.debug());
                break;
            }
            MessageView::Info(msg) => println!("{:?}", msg.debug()),
            MessageView::Progress(msg) => println!("{:?}", msg.message()),
            _ => (),
        }
    }

    pipeline.set_state(gstreamer::State::Null).unwrap();

}