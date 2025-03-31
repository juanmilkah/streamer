use std::{io::Write, net::TcpListener};

use opencv::{
    core::{Mat, Vector},
    videoio::{self, VideoCaptureTrait},
};

fn main() -> Result<(), ()> {
    let mut cam = opencv::videoio::VideoCapture::new(0, videoio::CAP_ANY)
        .expect("Failed to get video capture");
    let mut frame = Mat::default();
    let mut buf = Vector::new();

    let listener = TcpListener::bind("0.0.0.0:7979").expect("Failed to bind to port");
    println!("Server listening at port 0.0.0.0:7979");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary=frame\r\n\r\n"
                );
                stream
                    .write_all(response.as_bytes())
                    .map_err(|err| eprintln!("ERROR: {}", err))?;

                loop {
                    let image_data = format!(
                        "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                        buf.len()
                    );
                    cam.read(&mut frame).expect("Failed to read frame");
                    buf.clear();
                    let _ = opencv::imgcodecs::imencode(".jpg", &frame, &mut buf, &Vector::new());
                    stream
                        .write_all(image_data.as_bytes())
                        .map_err(|err| eprintln!("ERROR writing image data: {}", err))?;
                    stream
                        .write_all(buf.as_slice())
                        .map_err(|err| eprintln!("ERROR writing frame buf: {}", err))?;
                    // let _ = stream
                    //     .write_all(b"\r\n")
                    //     .map(|err| eprintln!("ERROR writing end of frame: {:?}", err));
                    stream.flush().unwrap();
                }
            }
            Err(e) => eprintln!("failed to get connection: {}", e),
        }
    }
    Ok(())
}
