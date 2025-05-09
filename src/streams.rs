use gstreamer::{MessageView, Pipeline, glib, prelude::*};
use log::{debug, error, info};
use tokio::sync::mpsc::Sender;

/// A structure defining a frame from a stream.
/// This should be the same thing returned from all
/// streams, in order to create an interface to
/// stream products
pub struct StreamFrame {
    pub source: String,
    pub data: Vec<u8>,
}

/// Interface for any stream.
pub trait Stream {
    /// The main function for running a stream.
    /// Each frame from the stream is to be sent through a
    /// sender in order to be able to process the frames from
    /// either a synchronous or asynchronous function
    fn stream(&self, tx: &Sender<StreamFrame>);
}

/// An RTSP Stream.
pub struct RTSPStream {
    pub stream_name: String,
    pub rtsp_uri: String,
}

impl Stream for RTSPStream {
    fn stream(&self, tx: &Sender<StreamFrame>) {
        info!(
            "[{}] Beginning stream from {}",
            &self.stream_name, &self.rtsp_uri
        );

        let pipeline = match self._setup_gstreamer_pipeline(&self.stream_name, &self.rtsp_uri, &tx)
        {
            Ok(pipeline) => pipeline,
            Err(bool_error) => {
                error!("[{}], {}", &self.stream_name, bool_error.message);
                return;
            }
        };

        match self._iter_on_bus(&pipeline) {
            Ok(()) => info!("[{}] Stream finished playing", &self.stream_name),
            Err(error_message) => {
                error!("[{}] {}", &self.stream_name, error_message);
                return;
            }
        }

        debug!("[{}] Exiting thread...", &self.stream_name);
    }
}

impl RTSPStream {
    fn _iter_on_bus(&self, pipeline: &Pipeline) -> Result<(), String> {
        pipeline.set_state(gstreamer::State::Playing).unwrap();

        let bus = pipeline.bus().unwrap();
        for msg in bus.iter_timed(None) {
            match msg.view() {
                MessageView::Eos(_) => break,
                MessageView::Error(err) => {
                    pipeline.set_state(gstreamer::State::Null).unwrap();
                    return Err(format!("Error: {:?}", err.error().message()));
                }
                MessageView::Progress(msg) => {
                    debug!("{:#?}", msg.structure().unwrap().value("text").unwrap());
                }
                _ => (),
            }
        }

        pipeline.set_state(gstreamer::State::Null).unwrap();
        Ok(())
    }

    fn _setup_gstreamer_pipeline(
        &self,
        location_name: &str,
        rtsp_location: &str,
        tx: &Sender<StreamFrame>,
    ) -> Result<gstreamer::Pipeline, glib::BoolError> {
        gstreamer::init().unwrap();

        let pipeline = gstreamer::Pipeline::default();
        let rtspsrc = gstreamer::ElementFactory::make("rtspsrc")
            .property("location", rtsp_location)
            .build()?;
        let depay = gstreamer::ElementFactory::make("rtph265depay").build()?;
        let parse = gstreamer::ElementFactory::make("h265parse").build()?;
        let decode = gstreamer::ElementFactory::make("avdec_h265").build()?;
        let videoconvert = gstreamer::ElementFactory::make("videoconvert").build()?;
        let jpegenc = gstreamer::ElementFactory::make("jpegenc").build()?;

        let appsink = gstreamer_app::AppSink::builder()
            .caps(&gstreamer::Caps::builder("image/jpeg").build())
            .build();

        let location_name_clone = location_name.to_string();
        let tx_clone = tx.clone();
        appsink.set_callbacks(
            gstreamer_app::AppSinkCallbacks::builder()
                .new_sample(move |appsink| {
                    let frame = appsink
                        .pull_sample()
                        .map_err(|_| gstreamer::FlowError::Eos)
                        .unwrap();
                    let buffer = frame.buffer().unwrap();

                    let map = buffer.map_readable().unwrap();
                    let data = map.as_slice().to_vec();

                    let frame = StreamFrame {
                        source: location_name_clone.clone(),
                        data: data,
                    };

                    tx_clone.blocking_send(frame).unwrap();
                    Ok(gstreamer::FlowSuccess::Ok)
                })
                .build(),
        );

        pipeline
            .add_many([
                &rtspsrc,
                &depay,
                &parse,
                &decode,
                &videoconvert,
                &jpegenc,
                &appsink.upcast_ref(),
            ])
            .unwrap();
        gstreamer::Element::link_many([
            &depay,
            &parse,
            &decode,
            &videoconvert,
            &jpegenc,
            &appsink.upcast_ref(),
        ])
        .unwrap();

        let location_name_clone_2 = location_name.to_string();
        rtspsrc.connect_pad_added(move |_, src_pad| {
            let sink_pad = depay.static_pad("sink").unwrap();
            let caps_string = src_pad.current_caps().unwrap().to_string();

            if sink_pad.is_linked()
                || src_pad.is_linked()
                || !caps_string.contains("media=(string)video")
            {
                return;
            }

            match src_pad.link(&sink_pad) {
                Ok(_) => (),
                Err(e) => panic!(
                    "[{}] Could not link rtspsrc to `depay` due to the following error: {:?}",
                    location_name_clone_2, e
                ),
            };
        });

        Ok(pipeline)
    }
}
