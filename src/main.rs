mod components;

use std::time;
use std::fs;
use std::path::PathBuf;
use std::net::SocketAddr;

use log::error;
use winit::{event::Event, event_loop::ControlFlow};
use clap::{arg, command, value_parser, ArgAction, Command};

use organum::error::Error;
use components::{build_talea, add_tty, TPS_PATH};

fn main() -> Result<(), Error> {

    let frequency = 10_000_000; //TODO: add to clap
    
    let matches = command!()

        .arg(
            arg!(
                -s --server <ADDR> "Sets a serial terminal and waits to connect to the server specified"
            )
            .required(false)
            .value_parser(value_parser!(String))
        )
        .arg(arg!(
            -d --debug ... "Turn the external debugger on"
        )
        .action(ArgAction::SetTrue)
        .required(false)
        )
        .arg(arg!([bin] "Binary image to bootstrap the system (a BIOS of sorts)")
        .required(true)
        .value_parser(value_parser!(PathBuf))
        )  
        .subcommand(
            Command::new("tps")
                .about("inserts a Tps device into the system")
                .arg(arg!(-l --list "lists the available Tps slots").action(ArgAction::SetTrue))
                .arg(arg!([path] "inserts the Tps device at the specified slot"))
                .arg(arg!([slot] "inserts the Tps device at the specified slot"))
        )
    .get_matches();
    
    let bin = matches.get_one::<PathBuf>("bin").unwrap();
    let ip = matches.get_one::<String>("server");
    let debug = matches.get_one::<bool>("debug");

    if let Some(matches) = matches.subcommand_matches("tps") {

        if *matches.get_one::<bool>("list").unwrap() {
            for file in fs::read_dir(TPS_PATH).unwrap() {
                let file = file.unwrap();
                println!("{}", file.file_name().into_string().unwrap())
            }
        }

        let tps = matches.get_one::<PathBuf>("path");

        if let Some(tps) = tps {
            let slot = matches.get_one::<PathBuf>("slot").expect("No slot specified");
            fs::copy(tps, slot).expect("Failed to copy Tps device");
        }
    }

    let socket: SocketAddr = ip.unwrap_or(&String::from("127.0.0.1:65432")).parse().unwrap();

    let mut talea = build_talea(bin, socket.ip(), socket.port(), *debug.unwrap())?;
    talea.server_run(); //TODO: The server starts as soon as it's created. Look for a workaround
    add_tty(&mut talea.system, talea.tty).expect("Failed to add tty");

    let mut d = false;
    if let Some(&true) = debug {
        println!("Debugger enabled.");
        talea.system.enable_debugging();
        d = true;
    }


    talea.event_loop.run(move |event, _, control_flow| {
        let now = time::Instant::now();
        
        if let Event::RedrawRequested(_) = event {

            if let Err(err) = talea.video.screen.render() {
                error!("pixels.render() failed: {err}");
                *control_flow = ControlFlow::Exit;
                return;
            }
        }
        
        // Handle input events
        if talea.input.update(&event) {
            // Close events
            if talea.input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = talea.input.window_resized() {
                if let Err(err) = talea.video.screen.framebuffer.resize_surface(size.width, size.height) {
                    error!("pixels.resize_surface() failed: {err}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            // Update internal state and request a redraw

            talea.window.request_redraw();
        }

        talea.video.update(&event, &talea.system, &talea.window);

        if d {
            let elapsed = now.elapsed().as_millis();
            println!("Frame took: {}ms to render", elapsed);
        }
        
        let now = time::Instant::now();
        let ns = 16_000_005; // 16ms
        talea.system.run_for(ns);

        if d {
            let elapsed = now.elapsed().as_millis();
            println!("Cpu took: {}ms to run {} ns", elapsed, ns);
        }
    });
}



